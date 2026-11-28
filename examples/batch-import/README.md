# Batch Import

Run this CLI from the repository root to see which rows survive when a worker
stops in the middle of a CSV import.

```sh
cargo run --locked -p yetlet-batch-import
```

The command generates five tray weigh-ins in a temporary directory. It first
imports them without interruption. Two further runs stop a child process on
opposite sides of the second SQLite commit. The parent reads back the database
after each child exits, then starts a fresh worker against the same input and
database. Each restart must match the uninterrupted run on every
`(row_no, tray, grams)` tuple and the final checkpoint.

| Stop at row 4 | Durable rows | Last committed CSV row | Restart begins at |
|---|---|---|---|
| Immediately before commit | 1–2 | 2 | Row 3 |
| Immediately after commit | 1–4 | 4 | Row 5 |

Each worker inserts up to two rows and updates the checkpoint in the same
transaction. It reports the selected boundary to its parent and waits there.
The parent uses Yetlet Core composition to signal and reap that child before
reading SQLite. The demo exits successfully after both comparisons and removes
its temporary files.

For a persistent import, pass `import <csv> <database>` after Cargo's `--`.
The input format is a `tray,grams` header followed by two unsigned decimal
integers per line. Use one writer and the same unchanged CSV for every restart.
The checkpoint counts data rows from one and excludes the header.
