CREATE TABLE issues (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    title   TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 120),
    body    TEXT NOT NULL CHECK (length(body) <= 4000),
    status  TEXT NOT NULL DEFAULT 'open'
            CHECK (status IN ('open', 'in_progress', 'closed')),
    version INTEGER NOT NULL DEFAULT 1 CHECK (version >= 1)
);
