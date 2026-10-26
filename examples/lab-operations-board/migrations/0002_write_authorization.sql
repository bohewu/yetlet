CREATE TABLE write_authority (
    account TEXT NOT NULL CHECK(account = 'alice'),
    project INTEGER NOT NULL CHECK(project = 1),
    write_enabled INTEGER NOT NULL CHECK(write_enabled IN (0,1)),
    generation INTEGER NOT NULL CHECK(generation > 0 AND (generation < 9223372036854775807 OR write_enabled = 0)),
    PRIMARY KEY(account, project)
) STRICT;
INSERT INTO write_authority(account,project,write_enabled,generation) VALUES('alice',1,1,1);
