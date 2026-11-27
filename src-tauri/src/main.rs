#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod index;
mod recovery;
mod store;
#[cfg(test)]
mod tests;
mod text_index;
mod thumbnails;
mod transfer;

use std::{fs, path::PathBuf, sync::Mutex};
use store::{Db, Result};
use tauri::{ipc::Response, Manager, State};

#[tauri::command]
async fn pdf(app: tauri::AppHandle, id: i64) -> Result<Response> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(Response::new(transfer::read_pdf(&app.state::<Db>(), id)?))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn add_library(app: tauri::AppHandle, path: String) -> Result<i64> {
    let root = fs::canonicalize(path)?;
    if !root.is_dir() {
        return Err("Choose an existing folder.".into());
    }
    let id = {
        let db = app.state::<Db>();
        let db = db.lock()?;
        db.query_row(
            "INSERT INTO libraries(root) VALUES (?) ON CONFLICT(root) DO UPDATE SET active=1 RETURNING id",
            [root.to_string_lossy().as_ref()], |row| row.get(0)
        )?
    };
    app.state::<index::Queue>().schedule(id, root)?;
    Ok(id)
}

#[tauri::command]
fn remove_library(id: i64, db: State<Db>, queue: State<index::Queue>) -> Result<()> {
    db.lock()?
        .execute("UPDATE libraries SET active=0 WHERE id=?", [id])?;
    queue.remove(id)?;
    Ok(())
}

#[tauri::command]
fn write_export(path: PathBuf, content: String) -> Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| std::io::Write::write_all(&mut file, content.as_bytes()))?;
    Ok(())
}

#[tauri::command]
fn updates_enabled(app: tauri::AppHandle) -> bool {
    app.config().plugins.0.contains_key("updater")
}

fn data_path(app: &tauri::AppHandle) -> Result<PathBuf> {
    match std::env::var_os("PDFVIEWER_DATA_DIR") {
        Some(path) => Ok(fs::canonicalize(path)?),
        None => app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string().into()),
    }
}

fn main() {
    let context = tauri::generate_context!();
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    if context.config().plugins.0.contains_key("updater") {
        builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
    }
    builder
        .setup(|app| {
            let path = data_path(app.handle())?;
            fs::create_dir_all(&path)?;
            let db = store::open(&path.join("library.sqlite"))?;
            app.manage(Db(Mutex::new(db)));
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                if let Err(error) = transfer::automatic_backup(&handle.state::<Db>(), &path) {
                    eprintln!("Metadata backup failed: {error}");
                }
            });
            let queue = index::Queue::start(app.handle().clone())?;
            for library in store::get_libraries(app.state::<Db>())? {
                queue.schedule(library.id, library.root.into())?;
            }
            app.manage(queue);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            updates_enabled,
            store::recent_pdfs,
            store::clear_recent_pdfs,
            store::mark_opened,
            pdf,
            add_library,
            remove_library,
            write_export,
            store::get_libraries,
            store::children,
            store::search,
            store::notes,
            store::save_note,
            store::delete_note,
            store::reading,
            store::save_reading,
            store::tags,
            store::set_tag,
            store::related,
            store::link,
            store::backup,
            store::restore,
            transfer::import_annotations,
            text_index::next_pdf,
            text_index::index_pages,
            thumbnails::get_thumbnail,
            thumbnails::put_thumbnail,
            thumbnails::clear_thumbnails,
            recovery::locate_pdf,
            recovery::forget_pdf,
            store::get_entry,
            store::graph_path,
            store::missing_entries
        ])
        .run(context)
        .expect("Could not start PDF Viewer");
}
