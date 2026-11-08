//! Only these predefined selections cross the native/browser boundary.
use crate::style_names::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub dark: bool,
    pub compact: bool,
    pub second: bool,
}

impl Selection {
    pub fn from_values(theme: &str, density: &str, selected: &str) -> Self {
        Self {
            dark: theme == "dark",
            compact: density == "compact",
            second: selected == "second",
        }
    }
    pub fn theme(self) -> &'static str {
        if self.dark { "dark" } else { "light" }
    }
    pub fn density(self) -> &'static str {
        if self.compact {
            "compact"
        } else {
            "comfortable"
        }
    }
    pub fn selected(self) -> &'static str {
        if self.second { "second" } else { "first" }
    }
    pub fn classes(self) -> String {
        format!(
            "{SHELL} {} {}",
            if self.dark { DARK } else { LIGHT },
            if self.compact { COMPACT } else { COMFORTABLE }
        )
    }
    pub fn href(self, path: &str, anchor: &str) -> String {
        format!(
            "{path}?theme={}&density={}&selected={}#{anchor}",
            self.theme(),
            self.density(),
            self.selected()
        )
    }
    pub fn change(self, kind: &str, value: &str) -> Self {
        match (kind, value) {
            ("theme", "light" | "dark") => Self {
                dark: value == "dark",
                ..self
            },
            ("density", "comfortable" | "compact") => Self {
                compact: value == "compact",
                ..self
            },
            ("selected", "first" | "second") => Self {
                second: value == "second",
                ..self
            },
            _ => self,
        }
    }
    pub fn value(self, kind: &str) -> &'static str {
        match kind {
            "theme" => self.theme(),
            "density" => self.density(),
            _ => self.selected(),
        }
    }
}
