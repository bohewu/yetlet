mod import;

use std::{
    env, fs,
    io::{self, BufRead, BufReader},
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use import::{AFTER, BEFORE};
use yetlet::composition::{Builder, Completion, WorkerCompletion};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const CSV: &str = "tray,grams\n11,240\n11,265\n12,301\n13,228\n13,251\n";

fn worker(csv: &Path, db: &Path) -> Result<Command> {
    let mut cmd = Command::new(env::current_exe()?);
    cmd.arg("import").arg(csv).arg(db);
    Ok(cmd)
}

fn finish_import(csv: &Path, db: &Path) -> Result<()> {
    let status = worker(csv, db)?.status()?;
    if !status.success() {
        return Err(format!("worker exited with {status}").into());
    }
    Ok(())
}

fn interrupt(csv: &Path, db: &Path, boundary: &str) -> Result<()> {
    let mut child = worker(csv, db)?
        .arg(boundary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().expect("configured piped stdout");
    let mut group = Builder::new()
        .required_worker(
            "import",
            child,
            |_| Ok(()), // This worker accepts only its initial CSV.
            |child| {
                child.kill().inspect_err(|_| {
                    // Release the worker through EOF if signalling failed.
                    child.stdin.take();
                })
            },
            |child: &mut Child, _| match child.try_wait()? {
                None => Ok(WorkerCompletion::Pending),
                Some(status) => {
                    println!("Worker {} exited with {status}", child.id());
                    let outcome = if status.success() {
                        Err(io::Error::other("expected a terminated worker"))
                    } else {
                        Ok(())
                    };
                    Ok(WorkerCompletion::Joined(outcome))
                }
            },
        )
        .build();

    let mut marker = String::new();
    let read = BufReader::new(stdout).read_line(&mut marker);
    // Reap the original child even when reading its marker failed.
    group.request_stop();
    while group.complete_until(Instant::now()) == Completion::Pending {
        thread::sleep(Duration::from_millis(1));
    }
    read?;
    if marker.trim_end() != format!("{boundary} row=4") {
        return Err(format!("worker missed {boundary} at row 4").into());
    }
    if let Some(error) = group.failures().primary() {
        return Err(error.to_string().into());
    }
    Ok(())
}

fn demo() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let csv = dir.path().join("weights.csv");
    fs::write(&csv, CSV)?;
    let reference_db = dir.path().join("reference.sqlite");
    finish_import(&csv, &reference_db)?;
    let reference = import::snapshot(&reference_db)?;
    if reference.last_row != 5 || reference.rows.len() != 5 {
        return Err("reference import did not finish all five rows".into());
    }
    println!("Uninterrupted: {reference:?}");

    for (boundary, expected_rows) in [(BEFORE, 2), (AFTER, 4)] {
        let db = dir.path().join(format!("{boundary}.sqlite"));
        interrupt(&csv, &db, boundary)?;
        let saved = import::snapshot(&db)?;
        println!("{boundary} at row 4: {saved:?}");
        if usize::try_from(saved.last_row)? != expected_rows
            || saved.rows != reference.rows[..expected_rows]
        {
            return Err(format!("unexpected durable state at {boundary}").into());
        }
        finish_import(&csv, &db)?;
        if import::snapshot(&db)? != reference {
            return Err(format!("resumed {boundary} result differs from reference").into());
        }
        println!("Resumed {boundary}: all rows and checkpoint match.");
    }
    dir.close()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match args.as_slice() {
        [] => demo(),
        [cmd, csv, db] if cmd == "import" => import::run(csv.as_ref(), db.as_ref(), None),
        [cmd, csv, db, stop] if cmd == "import" && (stop == BEFORE || stop == AFTER) => {
            import::run(csv.as_ref(), db.as_ref(), stop.to_str())
        }
        _ => Err("usage: yetlet-batch-import [import <csv> <database>]".into()),
    }
}
