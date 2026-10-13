CREATE TABLE work_items (
    id INTEGER PRIMARY KEY,
    project INTEGER NOT NULL CHECK(project IN (1,2)),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('open','active','blocked','closed')),
    priority TEXT NOT NULL CHECK(priority IN ('low','normal','high')),
    assignee TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1 CHECK(version >= 1),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch())
) STRICT;
CREATE INDEX work_project_updated ON work_items(project,updated_at DESC,id DESC);
CREATE INDEX work_project_status ON work_items(project,status,id);
CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id INTEGER NOT NULL REFERENCES work_items(id),
    project INTEGER NOT NULL CHECK(project IN (1,2)),
    version INTEGER NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('create','update')),
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    occurred_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(item_id,version)
) STRICT;
CREATE INDEX history_project ON history(project,id);
