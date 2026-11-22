use crate::{
    index::fingerprint,
    store::{Db, Result},
};
use rusqlite::{params, Connection, OptionalExtension};
use std::{fs, path::PathBuf, time::UNIX_EPOCH};
use tauri::State;

#[tauri::command]
pub fn locate_pdf(db: State<Db>, id: i64, path: PathBuf, confirm_mismatch: bool) -> Result<bool> {
    let (root, old_fingerprint) = db.lock()?.query_row(
        "SELECT l.root,e.fingerprint FROM entries e JOIN libraries l ON l.id=e.library
         WHERE e.id=? AND e.folder=0 AND e.missing=1 AND l.active=1",
        [id],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
    )?;
    let path = fs::canonicalize(path)?;
    if !path.starts_with(fs::canonicalize(&root)?)
        || !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    {
        return Err("Choose a PDF inside this library.".into());
    }
    let metadata = fs::metadata(&path)?;
    if !metadata.is_file() {
        return Err("Choose a PDF file.".into());
    }
    let hash = fingerprint(&path, metadata.len())?;
    if old_fingerprint.as_deref() != Some(hash.as_str()) && !confirm_mismatch {
        return Ok(false);
    }
    let parent = path
        .parent()
        .ok_or("The PDF must be in a library folder.")?
        .to_string_lossy();
    let name = path
        .file_name()
        .ok_or("The file has no name.")?
        .to_string_lossy();
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;
    let mut db = db.lock()?;
    let tx = db.transaction()?;
    let library: i64 = tx.query_row(
        "SELECT library FROM entries WHERE id=? AND folder=0 AND missing=1",
        [id],
        |r| r.get(0),
    )?;
    let path = path.to_string_lossy();
    let occupied: Option<i64> = tx
        .query_row(
            "SELECT id FROM entries WHERE library=? AND path=? AND id<>?",
            params![library, path, id],
            |r| r.get(0),
        )
        .optional()?;
    if let Some(occupied) = occupied {
        let has_data:bool=tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM notes WHERE entry=?1 UNION ALL SELECT 1 FROM reading WHERE entry=?1
             UNION ALL SELECT 1 FROM tags WHERE entry=?1 UNION ALL SELECT 1 FROM links WHERE a=?1 OR b=?1)",
            [occupied],|r|r.get(0)
        )?;
        if has_data {
            return Err("The chosen PDF already has saved metadata. Export the missing PDF’s annotations before importing them into the chosen PDF.".into());
        }
        tx.execute("DELETE FROM entries WHERE id=?", [occupied])?;
    }
    tx.execute("DELETE FROM pdf_index WHERE entry=?", [id])?;
    tx.execute("DELETE FROM pdf_text WHERE entry=?", [id])?;
    tx.execute("UPDATE entries SET path=?,parent=?,name=?,size=?,modified=?,fingerprint=?,missing=0 WHERE id=?",
        params![path,parent,name,metadata.len() as i64,modified,hash,id])?;
    tx.commit()?;
    Ok(true)
}

fn forget(conn: &mut Connection, id: i64) -> Result<()> {
    let tx = conn.transaction()?;
    let missing = tx
        .query_row(
            "SELECT missing FROM entries WHERE id=? AND folder=0",
            [id],
            |r| r.get::<_, bool>(0),
        )
        .optional()?;
    if missing != Some(true) {
        return Err("Only missing PDFs can be forgotten.".into());
    }
    tx.execute("DELETE FROM notes WHERE entry=?", [id])?;
    tx.execute("DELETE FROM reading WHERE entry=?", [id])?;
    tx.execute("DELETE FROM tags WHERE entry=?", [id])?;
    tx.execute("DELETE FROM links WHERE a=?1 OR b=?1", [id])?;
    tx.execute("DELETE FROM entries WHERE id=?", [id])?;
    tx.commit()?;
    Ok(())
}
#[tauri::command]
pub fn forget_pdf(db: State<Db>, id: i64) -> Result<()> {
    forget(&mut *db.lock()?, id)
}
