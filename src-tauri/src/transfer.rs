use crate::store::{Db, Note, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::Duration,
};
use tauri::State;

#[derive(Deserialize)]
struct Export {
    version: u32,
    document: Identity,
    notes: Vec<Note>,
}
#[derive(Deserialize)]
struct Identity {
    fingerprint: Option<String>,
    path: String,
}
#[derive(Deserialize)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
#[tauri::command]
pub fn import_annotations(
    db: State<Db>,
    id: i64,
    path: PathBuf,
    confirm_mismatch: bool,
) -> Result<Option<usize>> {
    const MAX_BYTES: u64 = 32 * 1024 * 1024;
    let mut data = Vec::new();
    fs::File::open(path)?
        .take(MAX_BYTES + 1)
        .read_to_end(&mut data)?;
    if data.len() as u64 > MAX_BYTES {
        return Err("Annotation export exceeds 32 MB.".into());
    }
    let export: Export =
        serde_json::from_slice(&data).map_err(|_| "This is not a valid annotation export.")?;
    import(&mut *db.lock()?, id, export, confirm_mismatch)
}

fn import(db: &mut Connection, id: i64, export: Export, confirm: bool) -> Result<Option<usize>> {
    if export.version != 1 || export.notes.len() > 10_000 {
        return Err("Unsupported annotation export format or size.".into());
    }
    let (path, fingerprint, folder) = db.query_row(
        "SELECT path,fingerprint,folder FROM entries WHERE id=?",
        [id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, bool>(2)?,
            ))
        },
    )?;
    let matches = if folder {
        export.document.path == path
    } else {
        fingerprint.is_some() && fingerprint == export.document.fingerprint
    };
    if !matches && !confirm {
        return Ok(None);
    }
    let tx = db.transaction()?;
    let mut count = 0;
    for note in &export.notes {
        validate(note, folder)?;
        count += tx.execute(
            "INSERT INTO notes(id,entry,kind,page,quote,content,color,rects) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(id) DO NOTHING",
            params![note.id,id,note.kind,note.page,note.quote,note.content,note.color,note.rects],
        )?;
    }
    tx.commit()?;
    Ok(Some(count))
}

fn validate(note: &Note, folder: bool) -> Result<()> {
    let rects: Vec<Rect> =
        serde_json::from_str(&note.rects).map_err(|_| "Invalid annotation coordinates.")?;
    let selection = matches!(note.kind.as_str(), "highlight" | "underline" | "selection");
    let positioned = selection || note.kind == "bookmark";
    if !matches!(
        note.kind.as_str(),
        "highlight" | "underline" | "selection" | "bookmark" | "document" | "folder"
    ) || folder != (note.kind == "folder")
        || (positioned && !note.page.is_some_and(|page| page > 0))
        || (!positioned && note.page.is_some())
        || selection == rects.is_empty()
        || rects.len() > 4096
        || rects.iter().any(|r| {
            r.x < -0.02
                || r.y < -0.02
                || r.w <= 0.0
                || r.h <= 0.0
                || r.x + r.w > 1.02
                || r.y + r.h > 1.02
        })
        || !note.color.starts_with('#')
        || note.color.len() != 7
        || !note.color[1..].bytes().all(|c| c.is_ascii_hexdigit())
        || note.id.is_empty()
        || note.id.len() > 100
    {
        return Err("The export contains an invalid annotation.".into());
    }
    Ok(())
}

pub fn automatic_backup(db: &Db, folder: &Path) -> Result<()> {
    let date = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
        / 86_400;
    let final_path = folder.join(format!("pdfviewer-daily-{date}.sqlite"));
    if final_path.exists() {
        return Ok(());
    }
    let candidate = folder.join("daily-backup.pending");
    db.lock()?.backup("main", &candidate, None)?;
    fs::rename(candidate, &final_path)?;
    let mut snapshots: Vec<_> = fs::read_dir(folder)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("pdfviewer-daily-") && name.ends_with(".sqlite")
                })
        })
        .collect();
    snapshots.sort_unstable_by_key(|path| {
        path.metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
    });
    for old in snapshots.iter().take(snapshots.len().saturating_sub(3)) {
        fs::remove_file(old)?;
    }
    Ok(())
}

pub fn read_pdf(db: &Db, id: i64) -> Result<Vec<u8>> {
    let row: Option<(String, String)> = db
        .lock()?
        .query_row(
            "SELECT e.path,l.root FROM entries e JOIN libraries l ON l.id=e.library
         WHERE e.id=? AND e.folder=0 AND e.missing=0 AND l.active=1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let (path, root) = row.ok_or("The PDF is missing from this library.")?;
    let canonical = fs::canonicalize(path)?;
    if !canonical.starts_with(fs::canonicalize(root)?)
        || !canonical
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
    {
        return Err("The selected PDF is outside its registered library.".into());
    }
    Ok(fs::read(canonical)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database() -> (tempfile::TempDir, Connection) {
        let temp = tempfile::tempdir().unwrap();
        let db = crate::store::open(&temp.path().join("metadata.sqlite")).unwrap();
        db.execute_batch(
            "INSERT INTO libraries(id,root) VALUES(1,'library');
             INSERT INTO entries(id,library,path,parent,name,folder,size,modified,fingerprint)
             VALUES(8,1,'library/book.pdf','library','book.pdf',0,12,0,'fixture')",
        )
        .unwrap();
        (temp, db)
    }

    fn snapshot(hash: &str, width: f64) -> Export {
        Export {
            version: 1,
            document: Identity {
                fingerprint: Some(hash.into()),
                path: "library/book.pdf".into(),
            },
            notes: vec![Note {
                id: "mark".into(),
                entry: 8,
                kind: "highlight".into(),
                page: Some(1),
                quote: "Selected text".into(),
                content: "An annotation".into(),
                color: "#ffbb00".into(),
                rects: format!(r#"[{{"x":0.1,"y":0.2,"w":{width},"h":0.05}}]"#),
            }],
        }
    }

    #[test]
    fn import_checks_identity_and_is_idempotent() {
        let (_temp, mut db) = database();
        assert_eq!(
            import(&mut db, 8, snapshot("mismatch", 0.25), false).unwrap(),
            None
        );
        assert_eq!(
            import(&mut db, 8, snapshot("fixture", 0.25), false).unwrap(),
            Some(1)
        );
        assert_eq!(
            import(&mut db, 8, snapshot("fixture", 0.25), false).unwrap(),
            Some(0)
        );
    }

    #[test]
    fn import_rolls_back_all_notes_on_invalid_geometry() {
        let (_temp, mut db) = database();
        let mut export = snapshot("fixture", 0.25);
        export.notes.extend(snapshot("fixture", 3.0).notes);
        assert!(import(&mut db, 8, export, false).is_err());
        assert_eq!(
            db.query_row("SELECT count(*) FROM notes", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn online_backup_copies_committed_wal_data() {
        let (temp, db) = database();
        db.execute("INSERT INTO tags VALUES(8,'reference')", [])
            .unwrap();
        let target = temp.path().join("snapshot.sqlite");
        db.backup("main", &target, None).unwrap();
        let copy = Connection::open(target).unwrap();
        assert_eq!(
            copy.query_row("SELECT label FROM tags", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "reference"
        );
    }
}
