use std::fs;
use std::path::Path;

use tempfile::tempdir;
use webp_to_gif::history::{HistoryEntry, HistoryStore};
use webp_to_gif::output::{
    default_output_dir_for_executable, ensure_output_dir, is_webp_file, unique_gif_path,
};

#[test]
fn default_output_dir_is_gif_folder_next_to_executable() {
    let base = tempdir().unwrap();
    let executable = base.path().join("WebP To GIF.exe");

    let output = default_output_dir_for_executable(&executable);

    assert_eq!(output, base.path().join("GIF"));
}

#[test]
fn ensure_output_dir_creates_missing_folder() {
    let base = tempdir().unwrap();
    let output = base.path().join("GIF");

    let created = ensure_output_dir(&output).unwrap();

    assert_eq!(created, output);
    assert!(output.is_dir());
}

#[test]
fn unique_gif_path_preserves_existing_files() {
    let base = tempdir().unwrap();
    let source = base.path().join("sample.webp");
    let output_dir = base.path().join("GIF");
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_dir.join("sample.gif"), b"existing").unwrap();

    let next = unique_gif_path(&source, &output_dir).unwrap();

    assert_eq!(next, output_dir.join("sample (1).gif"));
    assert_eq!(fs::read(output_dir.join("sample.gif")).unwrap(), b"existing");
}

#[test]
fn webp_file_filter_is_case_insensitive() {
    assert!(is_webp_file(Path::new("photo.webp")));
    assert!(is_webp_file(Path::new("photo.WEBP")));
    assert!(!is_webp_file(Path::new("photo.gif")));
    assert!(!is_webp_file(Path::new("photo")));
}

#[test]
fn history_round_trips_as_json() {
    let base = tempdir().unwrap();
    let history_path = base.path().join("history.json");
    let store = HistoryStore::new(history_path.clone());
    let entry = HistoryEntry {
        file_path: "/input/a.webp".into(),
        output_path: Some("/output/a.gif".into()),
        status: "completed".into(),
        message: "转换完成".into(),
        completed_at: "2026-06-17T12:00:00Z".into(),
    };

    store.save(&[entry.clone()]).unwrap();
    let loaded = store.load().unwrap();

    assert_eq!(loaded, vec![entry]);
    assert!(history_path.exists());
}
