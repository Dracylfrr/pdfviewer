CREATE TABLE libraries(
  id INTEGER PRIMARY KEY, root TEXT NOT NULL UNIQUE, active INTEGER NOT NULL DEFAULT 1
);
CREATE TABLE entries(
  id INTEGER PRIMARY KEY, library INTEGER NOT NULL REFERENCES libraries(id),
  path TEXT NOT NULL, parent TEXT NOT NULL, name TEXT NOT NULL,
  folder INTEGER NOT NULL, size INTEGER NOT NULL, modified INTEGER NOT NULL,
  fingerprint TEXT, missing INTEGER NOT NULL DEFAULT 0,
  UNIQUE(library,path)
);
CREATE INDEX entries_parent ON entries(library,parent,missing);
CREATE INDEX entries_fingerprint ON entries(library,fingerprint,missing);
CREATE TABLE reading(
  entry INTEGER PRIMARY KEY REFERENCES entries(id), page INTEGER NOT NULL,
  zoom REAL NOT NULL, mode TEXT NOT NULL, theme TEXT NOT NULL
);
CREATE TABLE notes(
  id TEXT PRIMARY KEY, entry INTEGER NOT NULL REFERENCES entries(id),
  kind TEXT NOT NULL, page INTEGER, quote TEXT NOT NULL, content TEXT NOT NULL,
  color TEXT NOT NULL, rects TEXT NOT NULL,
  created INTEGER NOT NULL DEFAULT (unixepoch()),
  modified INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX notes_entry ON notes(entry,page);
CREATE TABLE tags(entry INTEGER REFERENCES entries(id), label TEXT, PRIMARY KEY(entry,label));
CREATE TABLE links(a INTEGER REFERENCES entries(id), b INTEGER REFERENCES entries(id), PRIMARY KEY(a,b), CHECK(a<b));
PRAGMA user_version=1;
