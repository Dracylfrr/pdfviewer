use crate::{
    index::scan,
    store::{self, Db},
};
use rusqlite::params;
use std::{fs, sync::Mutex, time::Instant};

fn fixture() -> (tempfile::TempDir, Db, i64) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Books");
    fs::create_dir(&root).unwrap();
    let db = store::open(&temp.path().join("reader.sqlite")).unwrap();
    let id = db
        .query_row(
            "INSERT INTO libraries(root) VALUES (?) RETURNING id",
            [root.to_string_lossy().as_ref()],
            |r| r.get(0),
        )
        .unwrap();
    (temp, Db(Mutex::new(db)), id)
}

#[test]
fn migrates_and_reopens_with_notes_and_reading_state() {
    let (temp, db, id) = fixture();
    let root = temp.path().join("Books");
    fs::write(root.join("Book.pdf"), b"%PDF-1.7").unwrap();
    scan(&db, id, &root, true, |_, _| {}).unwrap();
    {
        let conn = db.lock().unwrap();
        let doc: i64 = conn
            .query_row("SELECT id FROM entries WHERE folder=0", [], |r| r.get(0))
            .unwrap();
        conn.execute("INSERT INTO notes(id,entry,kind,page,quote,content,color,rects) VALUES('n',?,'highlight',1,'text','comment','#ffffff','[{\"x\":0.1,\"y\":0.2,\"w\":0.3,\"h\":0.1}]')",[doc]).unwrap();
        conn.execute(
            "INSERT INTO reading(entry,page,zoom,mode,theme) VALUES (?,1,1.25,'single','Sepia')",
            [doc],
        )
        .unwrap();
    }
    drop(db);
    let db = store::open(&temp.path().join("reader.sqlite")).unwrap();
    let (content, rects): (String, String) = db
        .query_row("SELECT content,rects FROM notes WHERE id='n'", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    let geometry: serde_json::Value = serde_json::from_str(&rects).unwrap();
    assert_eq!(geometry[0]["x"], 0.1);
    assert_eq!(content, "comment");
    assert_eq!(
        db.query_row("SELECT zoom FROM reading", [], |r| r.get::<_, f64>(0))
            .unwrap(),
        1.25
    );
}

#[test]
fn incremental_scan_preserves_id_after_move_and_disappearance() {
    let (temp, db, id) = fixture();
    let root = temp.path().join("Books");
    let a = root.join("A");
    let b = root.join("B");
    fs::create_dir(&a).unwrap();
    fs::create_dir(&b).unwrap();
    let before = a.join("Old.pdf");
    let after = b.join("New.pdf");
    fs::write(&before, b"%PDF-1.7 stable").unwrap();
    scan(&db, id, &root, true, |_, _| {}).unwrap();
    let document: i64 = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT id FROM entries WHERE path=?",
            [before.to_string_lossy().as_ref()],
            |r| r.get(0),
        )
        .unwrap();
    fs::rename(&before, &after).unwrap();
    scan(&db, id, &b, false, |_, _| {}).unwrap();
    scan(&db, id, &a, false, |_, _| {}).unwrap();
    assert_eq!(
        db.lock()
            .unwrap()
            .query_row("SELECT path FROM entries WHERE id=?", [document], |r| {
                r.get::<_, String>(0)
            })
            .unwrap(),
        after.to_string_lossy()
    );
    fs::remove_file(&after).unwrap();
    scan(&db, id, &b, false, |_, _| {}).unwrap();
    assert!(db
        .lock()
        .unwrap()
        .query_row("SELECT missing FROM entries WHERE id=?", [document], |r| {
            r.get::<_, bool>(0)
        })
        .unwrap());
    fs::write(&after, b"%PDF-1.7 stable").unwrap();
    scan(&db, id, &b, false, |_, _| {}).unwrap();
    assert!(!db
        .lock()
        .unwrap()
        .query_row("SELECT missing FROM entries WHERE id=?", [document], |r| {
            r.get::<_, bool>(0)
        })
        .unwrap());
}

#[test]
fn indexes_ten_thousand_entries_in_a_wide_folder() {
    let (temp, db, id) = fixture();
    let root = temp.path().join("Books");
    for n in 0..10_000 {
        fs::write(root.join(format!("{n}.pdf")), n.to_string()).unwrap();
    }
    let start = Instant::now();
    scan(&db, id, &root, true, |_, _| {}).unwrap();
    println!(
        "Index scan of 10,000 small PDF-named files: {:?}",
        start.elapsed()
    );
    let start = Instant::now();
    let count: i64 = db
        .lock()
        .unwrap()
        .query_row(
            "SELECT count(*) FROM entries WHERE library=? AND parent=? AND missing=0",
            params![id, root.to_string_lossy()],
            |r| r.get(0),
        )
        .unwrap();
    println!("Indexed folder count: {:?}", start.elapsed());
    assert_eq!(count, 10_000);
}
