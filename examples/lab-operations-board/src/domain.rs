use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::Failure;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Principal {
    pub name: &'static str,
    pub project: i64,
    pub write: bool,
}
const USERS: [Principal; 3] = [
    Principal {
        name: "alice",
        project: 1,
        write: true,
    },
    Principal {
        name: "bob",
        project: 2,
        write: true,
    },
    Principal {
        name: "viewer",
        project: 1,
        write: false,
    },
];

#[cfg(any(feature = "oidc", feature = "valkey"))]
pub(crate) fn principal(name: &str) -> Option<Principal> {
    USERS.iter().copied().find(|user| user.name == name)
}

#[cfg(feature = "valkey")]
impl Serialize for Principal {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name)
    }
}
#[cfg(feature = "valkey")]
impl<'de> Deserialize<'de> for Principal {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        principal(&name).ok_or_else(|| serde::de::Error::custom("unknown local identity"))
    }
}

/// Synthetic local credentials, generated anew for each process. Not an IdP.
#[derive(Clone)]
pub struct DemoIdentities([String; 3]);
impl std::fmt::Debug for DemoIdentities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DemoIdentities { codes: [REDACTED] }")
    }
}
impl DemoIdentities {
    pub fn generate() -> Result<Self, Failure> {
        let mut codes = Vec::new();
        for _ in USERS {
            let mut data = [0u8; 32];
            getrandom::fill(&mut data).map_err(|_| Failure::Unavailable)?;
            let code: String = data.iter().map(|v| format!("{v:02x}")).collect();
            if codes.contains(&code) {
                return Err(Failure::Unavailable);
            }
            codes.push(code);
        }
        Ok(Self(codes.try_into().map_err(|_| Failure::Unavailable)?))
    }
    /// Terminal-only startup material. Callers must not put these codes in reports.
    pub fn terminal_codes(&self) -> [(&'static str, &str); 3] {
        std::array::from_fn(|i| (USERS[i].name, self.0[i].as_str()))
    }
    pub(crate) fn authenticate(&self, name: &str, code: &str) -> Option<Principal> {
        let found = USERS.iter().position(|u| u.name == name);
        let expected = &self.0[found.unwrap_or(0)];
        let mut candidate = [0u8; 64];
        if code.len() == candidate.len() {
            candidate.copy_from_slice(code.as_bytes());
        }
        let equal: bool = candidate.ct_eq(expected.as_bytes()).into();
        if equal && code.len() == 64 {
            found.map(|i| USERS[i])
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "valkey", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "valkey", serde(deny_unknown_fields))]
pub(crate) struct LoginSession {
    pub user: Option<Principal>,
    #[cfg(feature = "oidc")]
    #[cfg_attr(feature = "valkey", serde(default))]
    pub oidc: Option<crate::oidc::Pending>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Fields {
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assignee: String,
}
impl Default for Fields {
    fn default() -> Self {
        Self {
            title: String::new(),
            description: String::new(),
            status: "open".into(),
            priority: "normal".into(),
            assignee: String::new(),
        }
    }
}
impl Fields {
    pub fn errors(&self, project: i64) -> Vec<(&'static str, &'static str)> {
        let mut errors = Vec::new();
        if self.title.trim().is_empty()
            || self.title.chars().count() > 160
            || self.title.chars().any(char::is_control)
        {
            errors.push((
                "title",
                "Enter 1 to 160 characters without control characters.",
            ));
        }
        if self.description.chars().count() > 4_000
            || self
                .description
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t'))
        {
            errors.push((
                "description",
                "Use at most 4,000 characters and ordinary line breaks.",
            ));
        }
        if !matches!(
            self.status.as_str(),
            "open" | "active" | "blocked" | "closed"
        ) {
            errors.push(("status", "Select a valid status."));
        }
        if !matches!(self.priority.as_str(), "low" | "normal" | "high") {
            errors.push(("priority", "Select a valid priority."));
        }
        let operator = if project == 1 { "alice" } else { "bob" };
        if !self.assignee.is_empty() && self.assignee != operator {
            errors.push(("assignee", "Select an operator belonging to this project."));
        }
        errors
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct WorkItem {
    pub id: i64,
    pub project: i64,
    pub fields: Fields,
    pub version: i64,
    pub created_at: i64,
    pub updated_at: i64,
}
#[derive(Debug)]
pub(crate) struct History {
    pub version: i64,
    pub actor: String,
    pub action: String,
    pub title: String,
    pub status: String,
    pub occurred_at: i64,
}
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ListQuery {
    pub q: String,
    pub status: String,
    pub sort: String,
    pub page: u32,
    pub live: bool,
}
impl ListQuery {
    pub fn normalize(mut self) -> Result<Self, Failure> {
        if self.page == 0 {
            self.page = 1;
        }
        if self.sort.is_empty() {
            self.sort = "updated".into();
        }
        if self.page > 200
            || self.q.chars().count() > 100
            || self.q.chars().any(char::is_control)
            || !matches!(
                self.status.as_str(),
                "" | "open" | "active" | "blocked" | "closed"
            )
            || !matches!(self.sort.as_str(), "updated" | "title" | "priority")
        {
            return Err(Failure::Invalid);
        }
        Ok(self)
    }
    pub fn url(&self, project: i64, page: u32) -> String {
        let mut query = self.clone();
        query.page = page;
        format!(
            "/board/projects/{project}/items?{}",
            serde_urlencoded::to_string(query).expect("fixed scalar query")
        )
    }
}
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Snapshot {
    pub revision: i64,
    pub total: i64,
    pub open: i64,
    pub active: i64,
    pub blocked: i64,
    pub closed: i64,
}
#[derive(Debug)]
pub(crate) struct Listing {
    pub items: Vec<WorkItem>,
    pub matching: i64,
    pub snapshot: Snapshot,
}
