use std::fs;
use std::path::PathBuf;

pub fn notes_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("meera-notch")
        .join("notes.txt")
}

pub fn read_notes() -> String {
    let path = notes_path();
    if let Ok(text) = fs::read_to_string(&path) {
        if !text.trim().is_empty() {
            return text;
        }
    }
    "- Cek rilis ncharge v1.0 di GitHub\n- Tilawah QS. Al-Kahfi setiap Jumat pagi\n- Selesaikan review dokumen sebelum waktu Maghrib".to_string()
}

pub fn write_notes(content: &str) -> Result<(), String> {
    let path = notes_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("txt.tmp");
    fs::write(&tmp, content).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}
