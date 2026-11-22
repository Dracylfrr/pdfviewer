use crate::store::{Db, Result};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize)]
pub struct Candidate {
    pub id: i64,
    pub name: String,
    pub modified: i64,
    pub size: i64,
}
#[derive(Deserialize)]
pub struct PageText {
    page: i64,
    text: String,
}

#[tauri::command]
pub fn next_pdf(db: State<Db>, library: i64, after: i64) -> Result<Option<Candidate>> {
    Ok(db.lock()?.query_row(
        "SELECT e.id,e.name,e.modified,e.size FROM entries e LEFT JOIN pdf_index p
         ON p.entry=e.id AND p.modified=e.modified AND p.size=e.size
         WHERE e.library=? AND e.id>? AND e.folder=0 AND e.missing=0 AND p.entry IS NULL ORDER BY e.id LIMIT 1",
        params![library,after], |r| Ok(Candidate {id:r.get(0)?,name:r.get(1)?,modified:r.get(2)?,size:r.get(3)?}),
    ).optional()?)
}

#[tauri::command]
pub fn index_pages(
    db: State<Db>,
    item: Candidate,
    pages: Vec<PageText>,
    total: i64,
    start: bool,
    complete: bool,
) -> Result<bool> {
    let mut conn = db.lock()?;
    let tx = conn.transaction()?;
    let valid: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM entries WHERE id=? AND modified=? AND size=? AND missing=0)",
        params![item.id, item.modified, item.size],
        |r| r.get(0),
    )?;
    if !valid {
        return Ok(false);
    }
    if start {
        tx.execute("DELETE FROM pdf_index WHERE entry=?", [item.id])?;
        tx.execute("DELETE FROM pdf_text WHERE entry=?", [item.id])?;
    }
    for page in pages {
        tx.execute("INSERT INTO pdf_text VALUES (?,?,?) ON CONFLICT(entry,page) DO UPDATE SET text=excluded.text",
            params![item.id,page.page,page.text])?;
    }
    if complete {
        tx.execute(
            "INSERT OR REPLACE INTO pdf_index VALUES (?,?,?,?)",
            params![item.id, item.modified, item.size, total],
        )?;
    }
    tx.commit()?;
    Ok(true)
}
