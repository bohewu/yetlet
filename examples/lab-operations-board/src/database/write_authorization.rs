//! Private SQL-owned deny boundary for the opt-in application reference.
use super::*;

pub(super) struct WriteAuthorization {
    store: Arc<()>,
    account: &'static str,
    project: i64,
    generation: i64,
}

fn scope(principal: Principal) -> Result<(), Failure> {
    if principal.name != "alice" || principal.project != 1 || !principal.write {
        return Err(Failure::Forbidden);
    }
    Ok(())
}

pub(super) fn validate(c: &Connection) -> Result<(), Failure> {
    let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if version != 2 {
        return Err(Failure::Schema);
    }
    let mut columns = c
        .prepare("PRAGMA table_info(write_authority)")
        .map_err(|_| Failure::Schema)?;
    let actual = columns
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, i64>(5)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let expected = [
        ("account", "TEXT", 1, 1),
        ("project", "INTEGER", 1, 2),
        ("write_enabled", "INTEGER", 1, 0),
        ("generation", "INTEGER", 1, 0),
    ]
    .map(|(a, b, c, d)| (a.to_owned(), b.to_owned(), c, d));
    if actual != expected {
        return Err(Failure::Schema);
    }
    let strict: i64 = c.query_row("SELECT strict FROM pragma_table_list WHERE schema='main' AND name='write_authority' AND type='table'", [], |r| r.get(0)).map_err(|_| Failure::Schema)?;
    if strict != 1 {
        return Err(Failure::Schema);
    }
    let mut q =
        c.prepare("SELECT account,project,write_enabled,generation FROM write_authority LIMIT 2")?;
    let rows = q
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, i64>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if rows.len() > 1
        || rows.iter().any(|(a, p, e, g)| {
            a != "alice" || *p != 1 || !matches!(e, 0 | 1) || *g <= 0 || (*g == i64::MAX && *e != 0)
        })
    {
        return Err(Failure::Schema);
    }
    Ok(())
}

pub(super) fn prepare(c: &mut Connection, fresh: bool) -> Result<(), Failure> {
    if fresh {
        let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute_batch(include_str!("../../migrations/0001.sql"))?;
        tx.execute_batch(include_str!(
            "../../migrations/0002_write_authorization.sql"
        ))?;
        tx.pragma_update(None, "user_version", 2)?;
        tx.commit()?;
    } else {
        // Conservative reopen/restore policy: never promote a persisted allow.
        // Opening another strict Store also invalidates existing allowed tickets.
        let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
        validate(&tx)?;
        tx.execute("UPDATE write_authority SET write_enabled=0,generation=generation+1 WHERE write_enabled=1", [])?;
        tx.commit()?;
    }
    validate(c)
}

pub(super) fn check(
    c: &Connection,
    store: Option<&Arc<()>>,
    authorization: Option<WriteAuthorization>,
    principal: Principal,
) -> Result<(), Failure> {
    let Some(store) = store else {
        return if authorization.is_none() {
            Ok(())
        } else {
            Err(Failure::Forbidden)
        };
    };
    scope(principal)?;
    validate(c)?;
    let a = authorization.ok_or(Failure::Forbidden)?;
    if !Arc::ptr_eq(store, &a.store)
        || a.account != principal.name
        || a.project != principal.project
    {
        return Err(Failure::Forbidden);
    }
    let row = grant(c, principal)?;
    if row != Some((1, a.generation)) {
        return Err(Failure::Forbidden);
    }
    Ok(())
}

fn grant(c: &Connection, principal: Principal) -> Result<Option<(i64, i64)>, Failure> {
    Ok(c.query_row(
        "SELECT write_enabled,generation FROM write_authority WHERE account=?1 AND project=?2",
        params![principal.name, principal.project],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()?)
}

impl Database {
    pub(super) async fn authorize_write(
        &self,
        principal: Principal,
    ) -> Result<WriteAuthorization, Failure> {
        scope(principal)?;
        let store = self.write_authority.clone().ok_or(Failure::Configuration)?;
        let generation = self
            .run(move |c| {
                validate(c)?;
                match grant(c, principal)? {
                    Some((1, g)) if g < i64::MAX => Ok(g),
                    _ => Err(Failure::Forbidden),
                }
            })
            .await?;
        Ok(WriteAuthorization {
            store,
            account: principal.name,
            project: principal.project,
            generation,
        })
    }

    pub(crate) async fn set_write_enabled(
        &self,
        principal: Principal,
        enabled: bool,
    ) -> Result<(), Failure> {
        scope(principal)?;
        if self.write_authority.is_none() {
            return Err(Failure::Configuration);
        }
        self.run(move |c| {
            let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
            validate(&tx)?;
            let (current,generation) = grant(&tx,principal)?.ok_or(Failure::Forbidden)?;
            if enabled || current != 0 {
                let next = generation.checked_add(1).ok_or(Failure::Forbidden)?;
                if enabled && next == i64::MAX { return Err(Failure::Forbidden); }
                tx.execute("UPDATE write_authority SET write_enabled=?1,generation=?2 WHERE account=?3 AND project=?4", params![i64::from(enabled),next,principal.name,principal.project])?;
            }
            tx.commit()?;
            Ok(())
        }).await
    }
}

#[cfg(test)]
mod tests;
