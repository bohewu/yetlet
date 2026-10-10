CREATE TABLE notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    title       TEXT NOT NULL COLLATE NOCASE UNIQUE
                CHECK (length(title) BETWEEN 1 AND 120),
    body        TEXT NOT NULL CHECK (length(body) <= 4000),
    version     INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1),
    created_at  INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at  INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE TABLE note_audit (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    note_id       INTEGER NOT NULL,
    action        TEXT NOT NULL CHECK (action IN ('create', 'update', 'delete')),
    note_version  INTEGER NOT NULL CHECK (note_version >= 1),
    title         TEXT NOT NULL,
    created_at    INTEGER NOT NULL DEFAULT (unixepoch())
);

CREATE INDEX note_audit_note_id_idx ON note_audit(note_id, id);
