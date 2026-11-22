use crate::store::{Db, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
    time::{Duration, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};

enum Job {
    Add(i64, PathBuf),
    Remove(i64),
    Event(notify::Event),
}
pub struct Queue(Sender<Job>);
impl Queue {
    pub fn start(app: AppHandle) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let callback = tx.clone();
        let watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if let Ok(event) = event {
                if !matches!(event.kind, notify::EventKind::Access(_)) {
                    let _ = callback.send(Job::Event(event));
                }
            }
        })
        .map_err(|e| e.to_string())?;
        std::thread::spawn(move || {
            let mut watcher: RecommendedWatcher = watcher;
            let mut roots = HashMap::<i64, PathBuf>::new();
            while let Ok(job) = rx.recv() {
                let mut changed = HashSet::new();
                match job {
                    Job::Add(id, path) => {
                        let _ = watcher.watch(&path, RecursiveMode::Recursive);
                        roots.insert(id, path.clone());
                        if let Err(error) = scan_and_emit(&app, id, &path, true) {
                            let _ = app.emit("index-error", error.to_string());
                        }
                    }
                    Job::Remove(id) => {
                        if let Some(path) = roots.remove(&id) {
                            if !roots.values().any(|root| root == &path) {
                                let _ = watcher.unwatch(&path);
                            }
                        }
                    }
                    Job::Event(event) => {
                        changed.extend(event.paths);
                        while let Ok(job) = rx.recv_timeout(Duration::from_millis(180)) {
                            match job {
                                Job::Event(event) => changed.extend(event.paths),
                                other => {
                                    // Preserve control messages while coalescing notifications.
                                    match other {
                                        Job::Add(id, path) => {
                                            let _ = watcher.watch(&path, RecursiveMode::Recursive);
                                            roots.insert(id, path.clone());
                                            let _ = scan_and_emit(&app, id, &path, true);
                                        }
                                        Job::Remove(id) => {
                                            if let Some(path) = roots.remove(&id) {
                                                let _ = watcher.unwatch(&path);
                                            }
                                        }
                                        Job::Event(_) => unreachable!(),
                                    }
                                }
                            }
                            if changed.len() >= 128 {
                                break;
                            }
                        }
                    }
                }
                for (id, root) in &roots {
                    for path in changed.iter().filter(|path| path.starts_with(root)) {
                        if let Some(parent) =
                            path.parent().filter(|parent| parent.starts_with(root))
                        {
                            let _ = scan_and_emit(&app, *id, parent, false);
                        }
                    }
                }
            }
        });
        Ok(Self(tx))
    }
    pub fn schedule(&self, id: i64, path: PathBuf) -> Result<()> {
        self.0
            .send(Job::Add(id, path))
            .map_err(|e| e.to_string().into())
    }
    pub fn remove(&self, id: i64) -> Result<()> {
        self.0
            .send(Job::Remove(id))
            .map_err(|e| e.to_string().into())
    }
}

struct Item {
    path: String,
    name: String,
    folder: bool,
    size: i64,
    modified: i64,
    fingerprint: Option<String>,
}
pub(crate) fn fingerprint(path: &Path, size: u64) -> std::io::Result<String> {
    const SAMPLE: u64 = 64 * 1024;
    let mut file = fs::File::open(path)?;
    let mut hash = Sha256::new();
    hash.update(size.to_le_bytes());
    let offsets = if size > 3 * SAMPLE {
        vec![0, size / 2, size - SAMPLE]
    } else {
        vec![0]
    };
    for offset in offsets {
        file.seek(SeekFrom::Start(offset))?;
        let mut bytes = Vec::new();
        let limit = if size > 3 * SAMPLE { SAMPLE } else { size };
        (&mut file).take(limit).read_to_end(&mut bytes)?;
        hash.update(bytes);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn scan_and_emit(app: &AppHandle, id: i64, root: &Path, recursive: bool) -> Result<()> {
    scan(&app.state::<Db>(), id, root, recursive, |found, done| {
        let _ = app.emit("library-index", (id, found, done));
    })
}

pub(crate) fn scan(
    db: &Db,
    id: i64,
    root: &Path,
    recursive: bool,
    progress: impl Fn(usize, bool),
) -> Result<()> {
    let mut pending = vec![root.to_owned()];
    let mut found = 0;
    while let Some(dir) = pending.pop() {
        let parent = dir.to_string_lossy();
        let listing = match fs::read_dir(&dir) {
            Ok(listing) => listing,
            Err(e) if dir == root => {
                return Err(format!("Library folder is unavailable: {e}").into())
            }
            Err(_) => continue,
        };
        let mut items = Vec::new();
        let mut complete = true;
        for file in listing {
            let file = match file {
                Ok(file) => file,
                Err(_) => {
                    complete = false;
                    continue;
                }
            };
            let path = file.path();
            let kind = match file.file_type() {
                Ok(kind) => kind,
                Err(_) => {
                    complete = false;
                    continue;
                }
            };
            let folder = kind.is_dir();
            if !folder
                && !(kind.is_file()
                    && path
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("pdf")))
            {
                continue;
            }
            let meta = match file.metadata() {
                Ok(meta) => meta,
                Err(_) => {
                    complete = false;
                    continue;
                }
            };
            let size = meta.len() as i64;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |t| t.as_millis() as i64);
            let value = path.to_string_lossy().into_owned();
            let old = if folder {
                None
            } else {
                db.lock()?.query_row(
                    "SELECT fingerprint FROM entries WHERE library=? AND path=? AND size=? AND modified=?",
                    params![id,value,size,modified], |row| row.get::<_, Option<String>>(0)
                ).optional()?.flatten()
            };
            let hash = if folder {
                None
            } else {
                old.or_else(|| fingerprint(&path, meta.len()).ok())
            };
            if folder {
                let exists = db.lock()?.query_row(
                    "SELECT EXISTS(SELECT 1 FROM entries WHERE library=? AND path=? AND missing=0)",
                    params![id, value],
                    |row| row.get::<_, bool>(0),
                )?;
                if recursive || !exists {
                    pending.push(path);
                }
            } else {
                found += 1;
            }
            items.push(Item {
                path: value,
                name: file.file_name().to_string_lossy().into_owned(),
                folder,
                size,
                modified,
                fingerprint: hash,
            });
        }
        let mut connection = db.lock()?;
        let tx = connection.transaction()?;
        let registered: Option<String> = tx
            .query_row(
                "SELECT root FROM libraries WHERE id=? AND active=1",
                [id],
                |row| row.get(0),
            )
            .optional()?;
        if !registered.is_some_and(|path| dir.starts_with(path)) {
            break;
        }
        if complete {
            tx.execute(
                "UPDATE entries SET missing=1 WHERE library=? AND parent=?",
                params![id, parent],
            )?;
        }
        for item in &items {
            if let Some(hash) = &item.fingerprint {
                // Move only a unique candidate whose former location no longer exists.
                let candidates = {
                    let mut query = tx.prepare("SELECT id,path FROM entries WHERE library=? AND fingerprint=? AND path<>? LIMIT 2")?;
                    let rows = query.query_map(params![id, hash, item.path], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
                    })?;
                    rows.collect::<rusqlite::Result<Vec<_>>>()?
                };
                if let [(old_id, old_path)] = candidates.as_slice() {
                    if Path::new(old_path).try_exists().is_ok_and(|exists| !exists) {
                        tx.execute(
                            "UPDATE OR IGNORE entries SET path=? WHERE id=?",
                            params![item.path, old_id],
                        )?;
                    }
                }
            }
            tx.execute("INSERT INTO entries(library,path,parent,name,folder,size,modified,fingerprint) VALUES(?,?,?,?,?,?,?,?)
                ON CONFLICT(library,path) DO UPDATE SET parent=excluded.parent,name=excluded.name,folder=excluded.folder,
                size=excluded.size,modified=excluded.modified,fingerprint=excluded.fingerprint,missing=0",
                params![id,item.path,parent,item.name,item.folder,item.size,item.modified,item.fingerprint])?;
        }
        if complete {
            tx.execute("WITH RECURSIVE gone(path) AS (
                SELECT path FROM entries WHERE library=?1 AND parent=?2 AND folder=1 AND missing=1
                UNION SELECT e.path FROM entries e JOIN gone ON e.parent=gone.path WHERE e.library=?1 AND e.folder=1
            ) UPDATE entries SET missing=1 WHERE library=?1 AND parent IN gone", params![id,parent])?;
        }
        tx.commit()?;
        drop(connection);
        if pending.is_empty() || found % 200 == 0 {
            progress(found, pending.is_empty());
        }
    }
    Ok(())
}
