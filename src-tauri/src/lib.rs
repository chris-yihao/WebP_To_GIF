// Copyright (c) 2026 Chris_yihao.
// Author: Chris_yihao
// Time: 2026-06-17

use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use converter::convert_webp_to_gif;
use history::{HistoryEntry, HistoryStore};
use output::{
    default_output_dir_for_executable, ensure_output_dir, is_webp_file, unique_gif_path,
};
use serde::Serialize;
use settings::{Settings, SettingsStore};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_dialog::DialogExt;

pub mod converter;
pub mod history;
pub mod output;
pub mod settings;

#[derive(Debug)]
struct AppState {
    output_dir: Mutex<PathBuf>,
    settings_store: SettingsStore,
    history_store: HistoryStore,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionJob {
    file_path: String,
    status: String,
    progress: u8,
    output_path: Option<String>,
    error: Option<String>,
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn now_stamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}

fn emit_progress(app: &AppHandle, job: &ConversionJob) {
    let _ = app.emit("conversion-progress", job);
}

fn app_data_dir(app: &tauri::App) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("webp-to-gif"))
}

fn initial_output_dir() -> Result<PathBuf, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    Ok(default_output_dir_for_executable(&executable))
}

#[tauri::command]
fn get_output_dir(state: State<'_, AppState>) -> Result<String, String> {
    let output_dir = state
        .output_dir
        .lock()
        .map_err(|_| "无法读取输出目录".to_string())?
        .clone();
    Ok(path_to_string(output_dir))
}

#[tauri::command]
fn set_output_dir(path: String, state: State<'_, AppState>) -> Result<String, String> {
    let output_dir = ensure_output_dir(PathBuf::from(path).as_path())
        .map_err(|error| error.to_string())?;

    state
        .settings_store
        .save(&Settings {
            output_dir: output_dir.clone(),
        })
        .map_err(|error| error.to_string())?;

    *state
        .output_dir
        .lock()
        .map_err(|_| "无法保存输出目录".to_string())? = output_dir.clone();

    Ok(path_to_string(output_dir))
}

#[tauri::command]
async fn choose_output_dir(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let Some(folder) = app
        .dialog()
        .file()
        .set_title("选择 GIF 输出文件夹")
        .blocking_pick_folder()
    else {
        return Ok(None);
    };

    let path = folder
        .into_path()
        .map_err(|_| "无法使用这个文件夹".to_string())?;
    set_output_dir(path_to_string(path), state).map(Some)
}

#[tauri::command]
fn open_output_dir(state: State<'_, AppState>) -> Result<(), String> {
    let output_dir = state
        .output_dir
        .lock()
        .map_err(|_| "无法读取输出目录".to_string())?
        .clone();
    ensure_output_dir(&output_dir).map_err(|error| error.to_string())?;
    open::that(output_dir).map_err(|error| error.to_string())
}

#[tauri::command]
fn load_history(state: State<'_, AppState>) -> Result<Vec<HistoryEntry>, String> {
    state.history_store.load().map_err(|error| error.to_string())
}

#[tauri::command]
fn convert_files(
    app: AppHandle,
    paths: Vec<String>,
    output_dir: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ConversionJob>, String> {
    let selected_output_dir = match output_dir {
        Some(path) => ensure_output_dir(PathBuf::from(path).as_path())
            .map_err(|error| error.to_string())?,
        None => state
            .output_dir
            .lock()
            .map_err(|_| "无法读取输出目录".to_string())?
            .clone(),
    };
    ensure_output_dir(&selected_output_dir).map_err(|error| error.to_string())?;

    let mut jobs = Vec::new();
    let mut history = state.history_store.load().unwrap_or_default();

    for file_path in paths {
        let source = PathBuf::from(&file_path);
        let mut job = ConversionJob {
            file_path: file_path.clone(),
            status: "converting".into(),
            progress: 5,
            output_path: None,
            error: None,
        };
        emit_progress(&app, &job);

        if !is_webp_file(&source) {
            job.status = "failed".into();
            job.progress = 100;
            job.error = Some("请选择 WebP 文件".into());
            emit_progress(&app, &job);
            history.insert(0, history_entry(&job, "请选择 WebP 文件"));
            jobs.push(job);
            continue;
        }

        let result = unique_gif_path(&source, &selected_output_dir)
            .map_err(|error| error.to_string())
            .and_then(|output_path| {
                job.progress = 35;
                job.output_path = Some(path_to_string(output_path.clone()));
                emit_progress(&app, &job);
                convert_webp_to_gif(&source, &output_path)
                    .map(|_| output_path)
                    .map_err(|error| error.to_string())
            });

        match result {
            Ok(output_path) => {
                job.status = "completed".into();
                job.progress = 100;
                job.output_path = Some(path_to_string(output_path));
                emit_progress(&app, &job);
                history.insert(0, history_entry(&job, "转换完成"));
            }
            Err(error) => {
                job.status = "failed".into();
                job.progress = 100;
                job.error = Some(error.clone());
                emit_progress(&app, &job);
                history.insert(0, history_entry(&job, &error));
            }
        }

        jobs.push(job);
    }

    history.truncate(50);
    state
        .history_store
        .save(&history)
        .map_err(|error| error.to_string())?;

    Ok(jobs)
}

fn history_entry(job: &ConversionJob, message: &str) -> HistoryEntry {
    HistoryEntry {
        file_path: job.file_path.clone(),
        output_path: job.output_path.clone(),
        status: job.status.clone(),
        message: message.into(),
        completed_at: now_stamp(),
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data = app_data_dir(app);
            let settings_store = SettingsStore::new(app_data.join("settings.json"));
            let history_store = HistoryStore::new(app_data.join("history.json"));
            let default_output_dir = initial_output_dir()?;
            let settings = settings_store
                .load_or_default(default_output_dir)
                .map_err(|error| error.to_string())?;
            let output_dir = ensure_output_dir(&settings.output_dir)
                .map_err(|error| error.to_string())?;

            app.manage(AppState {
                output_dir: Mutex::new(output_dir),
                settings_store,
                history_store,
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_output_dir,
            set_output_dir,
            choose_output_dir,
            open_output_dir,
            load_history,
            convert_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
