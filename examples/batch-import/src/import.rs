use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

use rusqlite::{Connection, params};

use crate::Result;

pub const BEFORE: &str = "before-commit";
pub const AFTER: &str = "after-commit";

fn pause_at(selected: Option<&str>, phase: &str, row: i64) -> Result<()> {
    if selected == Some(phase) && row == 4 {
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{phase} row={row}")?;
        stdout.flush()?;
        // The parent holds the pipe open until it kills this process.
        std::io::stdin().read_exact(&mut [0])?;
        return Err("interrupted worker unexpectedly continued".into());
    }
    Ok(())
}

fn checkpoint(db: &Connection) -> rusqlite::Result<i64> {
    db.query_row("SELECT last_row FROM progress WHERE id = 1", [], |r| {
        r.get(0)
    })
}

pub fn run(csv: &Path, database: &Path, stop: Option<&str>) -> Result<()> {
    let mut lines = BufReader::new(File::open(csv)?).lines();
    if lines.next().transpose()?.as_deref() != Some("tray,grams") {
        return Err("expected CSV header tray,grams".into());
    }
    let mut db = Connection::open(database)?;
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS weights (
             row_no INTEGER PRIMARY KEY, tray INTEGER NOT NULL, grams INTEGER NOT NULL
         );
         CREATE TABLE IF NOT EXISTS progress (
             id INTEGER PRIMARY KEY CHECK(id = 1), last_row INTEGER NOT NULL
         );
         INSERT OR IGNORE INTO progress VALUES (1, 0);",
    )?;
    let mut last_row = checkpoint(&db)?;
    for _ in 0..last_row {
        lines.next().ok_or("checkpoint exceeds CSV length")??;
    }

    loop {
        let batch = lines
            .by_ref()
            .take(2)
            .collect::<std::io::Result<Vec<_>>>()?;
        if batch.is_empty() {
            return Ok(());
        }

        let tx = db.transaction()?;
        for line in batch {
            last_row += 1;
            let (tray, grams) = line.split_once(',').ok_or("expected two numeric fields")?;
            tx.execute(
                "INSERT INTO weights (row_no, tray, grams) VALUES (?1, ?2, ?3)",
                params![last_row, tray.parse::<u32>()?, grams.parse::<u32>()?],
            )?;
        }
        tx.execute("UPDATE progress SET last_row = ?1 WHERE id = 1", [last_row])?;
        pause_at(stop, BEFORE, last_row)?;
        tx.commit()?;
        pause_at(stop, AFTER, last_row)?;
    }
}

#[derive(Debug, PartialEq)]
pub struct Snapshot {
    pub last_row: i64,
    pub rows: Vec<(i64, u32, u32)>,
}

pub fn snapshot(database: &Path) -> Result<Snapshot> {
    let db = Connection::open(database)?;
    let last_row = checkpoint(&db)?;
    let mut query = db.prepare("SELECT row_no, tray, grams FROM weights ORDER BY row_no")?;
    let rows = query
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Snapshot { last_row, rows })
}
