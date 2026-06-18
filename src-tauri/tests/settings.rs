use tempfile::tempdir;
use webp_to_gif::settings::{Settings, SettingsStore};

#[test]
fn missing_settings_loads_default_output_dir() {
    let base = tempdir().unwrap();
    let store = SettingsStore::new(base.path().join("settings.json"));
    let default_output = base.path().join("GIF");

    let settings = store.load_or_default(default_output.clone()).unwrap();

    assert_eq!(settings.output_dir, default_output);
}

#[test]
fn selected_output_dir_persists_between_loads() {
    let base = tempdir().unwrap();
    let store = SettingsStore::new(base.path().join("settings.json"));
    let selected = base.path().join("custom");

    store
        .save(&Settings {
            output_dir: selected.clone(),
        })
        .unwrap();
    let settings = store.load_or_default(base.path().join("GIF")).unwrap();

    assert_eq!(settings.output_dir, selected);
}
