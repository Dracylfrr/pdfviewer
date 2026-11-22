use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct Error(String);
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<String> for Error {
    fn from(e: String) -> Self {
        Self(e)
    }
}
impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Self(e.into())
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self(e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, Error>;

pub struct Db(pub Mutex<Connection>);
impl Db {
    pub fn lock(&self) -> Result<MutexGuard<'_, Connection>> {
        self.0
            .lock()
            .map_err(|_| "Library database is unavailable.".into())
    }
}

pub const SCHEMA_VERSION: i64 = 2;
pub fn open(path: &Path) -> Result<Connection> {
    let mut db = Connection::open(path)?;
    let version: i64 = db.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err("This database requires a newer version of PDF Viewer.".into());
    }
    if version < SCHEMA_VERSION {
        if std::fs::metadata(path)?.len() > 0 {
            db.backup("main", path.with_extension("pre-migration.sqlite"), None)?;
        }
        let tx = db.transaction()?;
        if version == 0 {
            tx.execute_batch(include_str!("../migrations/001.sql"))?;
        }
        tx.execute_batch(include_str!("../migrations/002.sql"))?;
        tx.commit()?;
    }
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;")?;
    Ok(db)
}

#[derive(Serialize)]
pub struct Library {
    pub id: i64,
    pub root: String,
}
#[tauri::command]
pub fn get_libraries(db: State<Db>) -> Result<Vec<Library>> {
    let db = db.lock()?;
    let mut query = db.prepare("SELECT id,root FROM libraries WHERE active=1 ORDER BY id")?;
    let rows = query.query_map([], |row| {
        Ok(Library {
            id: row.get(0)?,
            root: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[derive(Serialize)]
pub struct Entry {
    id: i64,
    library: i64,
    path: String,
    parent: String,
    name: String,
    folder: bool,
    size: i64,
    modified: i64,
    missing: bool,
    fingerprint: Option<String>,
}
const COLUMNS: &str =
    "e.id,e.library,e.path,e.parent,e.name,e.folder,e.size,e.modified,e.missing,e.fingerprint";
fn entry(row: &Row<'_>) -> rusqlite::Result<Entry> {
    Ok(Entry {
        id: row.get(0)?,
        library: row.get(1)?,
        path: row.get(2)?,
        parent: row.get(3)?,
        name: row.get(4)?,
        folder: row.get(5)?,
        size: row.get(6)?,
        modified: row.get(7)?,
        missing: row.get(8)?,
        fingerprint: row.get(9)?,
    })
}

#[tauri::command]
pub fn children(db: State<Db>, library: i64, parent: String) -> Result<Vec<Entry>> {
    let db = db.lock()?;
    let mut query = db.prepare(&format!(
        "SELECT {COLUMNS} FROM entries e WHERE library=? AND parent=? AND (missing=0 OR folder=0)"
    ))?;
    let rows = query.query_map(params![library, parent], entry)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[derive(Serialize)]
pub struct Hit {
    entry: Entry,
    page: Option<i64>,
    snippet: String,
}
#[tauri::command]
pub fn search(db: State<Db>, term: String) -> Result<Vec<Hit>> {
    let db = db.lock()?;
    let mut query = db.prepare(&format!(
        "SELECT {COLUMNS}, NULL, e.name FROM entries e JOIN libraries l ON l.id=e.library
         WHERE l.active=1 AND (instr(lower(e.name),lower(?1))>0 OR EXISTS(
           SELECT 1 FROM tags WHERE entry=e.id AND instr(lower(label),lower(?1))>0))
         UNION ALL
         SELECT {COLUMNS}, n.page, trim(n.quote || ' ' || n.content)
         FROM notes n JOIN entries e ON e.id=n.entry JOIN libraries l ON l.id=e.library
         WHERE l.active=1 AND instr(lower(n.quote || ' ' || n.content),lower(?1))>0
         UNION ALL
         SELECT {COLUMNS}, t.page, substr(t.text,max(1,instr(lower(t.text),lower(?1))-30),120)
         FROM pdf_text t JOIN pdf_index p ON p.entry=t.entry JOIN entries e ON e.id=t.entry
         JOIN libraries l ON l.id=e.library
         WHERE l.active=1 AND e.missing=0 AND p.modified=e.modified AND p.size=e.size
         AND instr(lower(t.text),lower(?1))>0 LIMIT 250"
    ))?;
    let rows = query.query_map([term.trim()], |row| {
        Ok(Hit {
            entry: entry(row)?,
            page: row.get(10)?,
            snippet: row.get(11)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub entry: i64,
    pub kind: String,
    pub page: Option<i64>,
    pub quote: String,
    pub content: String,
    pub color: String,
    pub rects: String,
}
#[tauri::command]
pub fn notes(db: State<Db>, id: i64) -> Result<Vec<Note>> {
    let db = db.lock()?;
    let mut query = db.prepare("SELECT id,entry,kind,page,quote,content,color,rects FROM notes WHERE entry=? ORDER BY created,id")?;
    let rows = query.query_map([id], |r| {
        Ok(Note {
            id: r.get(0)?,
            entry: r.get(1)?,
            kind: r.get(2)?,
            page: r.get(3)?,
            quote: r.get(4)?,
            content: r.get(5)?,
            color: r.get(6)?,
            rects: r.get(7)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
#[tauri::command]
pub fn save_note(db: State<Db>, note: Note) -> Result<()> {
    db.lock()?.execute("INSERT INTO notes(id,entry,kind,page,quote,content,color,rects)
        VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET content=excluded.content,color=excluded.color,modified=unixepoch() WHERE notes.entry=excluded.entry",
        params![note.id,note.entry,note.kind,note.page,note.quote,note.content,note.color,note.rects])?;
    Ok(())
}
#[tauri::command]
pub fn delete_note(db: State<Db>, id: String) -> Result<()> {
    db.lock()?.execute("DELETE FROM notes WHERE id=?", [id])?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
pub struct Reading {
    page: i64,
    zoom: f64,
    mode: String,
    theme: String,
}
#[tauri::command]
pub fn reading(db: State<Db>, id: i64) -> Result<Option<Reading>> {
    use rusqlite::OptionalExtension;
    Ok(db
        .lock()?
        .query_row(
            "SELECT page,zoom,mode,theme FROM reading WHERE entry=?",
            [id],
            |r| {
                Ok(Reading {
                    page: r.get(0)?,
                    zoom: r.get(1)?,
                    mode: r.get(2)?,
                    theme: r.get(3)?,
                })
            },
        )
        .optional()?)
}
#[tauri::command]
pub fn save_reading(db: State<Db>, id: i64, state: Reading) -> Result<()> {
    db.lock()?.execute("INSERT INTO reading VALUES(?,?,?,?,?)
        ON CONFLICT(entry) DO UPDATE SET page=excluded.page,zoom=excluded.zoom,mode=excluded.mode,theme=excluded.theme",
        params![id,state.page,state.zoom,state.mode,state.theme])?;
    Ok(())
}

#[tauri::command]
pub fn tags(db: State<Db>, id: i64) -> Result<Vec<String>> {
    let db = db.lock()?;
    let mut query = db.prepare("SELECT label FROM tags WHERE entry=? ORDER BY label")?;
    let rows = query.query_map([id], |row| row.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
#[tauri::command]
pub fn set_tag(db: State<Db>, id: i64, label: String, remove: bool) -> Result<()> {
    let label = label.trim();
    if label.is_empty() || label.len() > 128 {
        return Err("Enter a tag between 1 and 128 bytes.".into());
    }
    let sql = if remove {
        "DELETE FROM tags WHERE entry=? AND label=?"
    } else {
        "INSERT OR IGNORE INTO tags VALUES(?,?)"
    };
    db.lock()?.execute(sql, params![id, label])?;
    Ok(())
}
#[tauri::command]
pub fn related(db: State<Db>, id: i64) -> Result<Vec<Entry>> {
    let db = db.lock()?;
    let mut query = db.prepare(&format!(
        "SELECT {COLUMNS} FROM links JOIN entries e ON e.id=CASE WHEN a=?1 THEN b ELSE a END
        JOIN libraries l ON l.id=e.library WHERE (a=?1 OR b=?1) AND l.active=1"
    ))?;
    let rows = query.query_map([id], entry)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
#[tauri::command]
pub fn link(db: State<Db>, a: i64, b: i64) -> Result<()> {
    if a == b {
        return Err("Choose a different document.".into());
    }
    db.lock()?.execute(
        "INSERT OR IGNORE INTO links VALUES(?,?)",
        params![a.min(b), a.max(b)],
    )?;
    Ok(())
}
#[tauri::command]
pub fn backup(db: State<Db>, path: PathBuf) -> Result<()> {
    std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)?;
    db.lock()?.backup("main", path, None)?;
    Ok(())
}
#[tauri::command]
pub fn restore(db: State<Db>, path: PathBuf) -> Result<()> {
    let source = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let version: i64 = source.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    let healthy: String = source.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if version != SCHEMA_VERSION || healthy != "ok" {
        return Err("The backup is incompatible or damaged.".into());
    }
    let mut db = db.lock()?;
    // Merge notes only is unsafe when document IDs differ. Restore the complete snapshot.
    let previous = PathBuf::from(db.path().ok_or("Cannot locate database.")?)
        .with_extension("pre-restore.sqlite");
    db.backup("main", previous, None)?;
    rusqlite::backup::Backup::new(&source, &mut db)?.run_to_completion(
        128,
        std::time::Duration::from_millis(10),
        None,
    )?;
    Ok(())
}
#[tauri::command]
pub fn get_entry(db: State<Db>, id: i64) -> Result<Entry> {
    Ok(db.lock()?.query_row(
        &format!("SELECT {COLUMNS} FROM entries e JOIN libraries l ON l.id=e.library WHERE e.id=? AND l.active=1"),
        [id], entry
    )?)
}

#[tauri::command]
pub fn missing_entries(db: State<Db>, library: i64) -> Result<Vec<Hit>> {
    let db = db.lock()?;
    let mut query=db.prepare(&format!("SELECT {COLUMNS} FROM entries e WHERE e.library=? AND e.folder=0 AND e.missing=1 ORDER BY e.name"))?;
    let rows = query.query_map([library], |row| {
        Ok(Hit {
            entry: entry(row)?,
            page: None,
            snippet: "Missing PDF".into(),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}
