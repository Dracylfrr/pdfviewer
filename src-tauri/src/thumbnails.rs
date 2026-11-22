use std::{fs, path::PathBuf};
use tauri::ipc::Response;

fn directory(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(crate::data_path(app)
        .map_err(|e| e.to_string())?
        .join("thumbnails"))
}

#[tauri::command]
pub fn get_thumbnail(
    app: tauri::AppHandle,
    id: i64,
    stamp: i64,
    page: u32,
) -> Result<Response, String> {
    let path = directory(&app)?.join(format!("{id}-{stamp}-{page}.webp"));
    Ok(Response::new(fs::read(path).unwrap_or_default()))
}

#[tauri::command]
pub fn put_thumbnail(
    app: tauri::AppHandle,
    id: i64,
    stamp: i64,
    page: u32,
    data: Vec<u8>,
) -> Result<(), String> {
    const MAX_THUMBNAILS: usize = 800;
    if data.len() > 32_768 || !data.starts_with(b"RIFF") || data.get(8..12) != Some(b"WEBP") {
        return Err("Invalid thumbnail.".into());
    }
    let dir = directory(&app)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::write(dir.join(format!("{id}-{stamp}-{page}.webp")), data).map_err(|e| e.to_string())?;
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "webp"))
        .collect();
    if entries.len() > MAX_THUMBNAILS {
        entries
            .sort_unstable_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok());
        for entry in entries.iter().take(entries.len() - MAX_THUMBNAILS + 50) {
            let _ = fs::remove_file(entry.path());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn clear_thumbnails(app: tauri::AppHandle) -> Result<(), String> {
    let dir = directory(&app)?;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "webp"))
        {
            fs::remove_file(entry.path()).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
