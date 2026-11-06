// Masque la console sous Windows en version release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bandua_core::{free_space, run, scan, verify_mhl, Event, Job, ScanInfo};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

#[derive(Default)]
struct AppState {
    cancel: Mutex<Option<Arc<AtomicBool>>>,
}

#[derive(serde::Serialize)]
struct DiskInfo {
    path: String,
    free: Option<u64>,
}

#[tauri::command]
async fn pick_folder(app: AppHandle) -> Option<String> {
    app.dialog().file().blocking_pick_folder().map(|p| p.to_string())
}

#[tauri::command]
async fn pick_mhl(app: AppHandle) -> Option<String> {
    app.dialog().file().add_filter("MHL", &["mhl"]).blocking_pick_file().map(|p| p.to_string())
}

#[tauri::command]
async fn scan_source(path: String) -> Result<ScanInfo, String> {
    tauri::async_runtime::spawn_blocking(move || scan(&PathBuf::from(path)).map(|(_, info)| info))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn disk_info(path: String) -> DiskInfo {
    DiskInfo { free: free_space(&PathBuf::from(&path)), path }
}

#[tauri::command]
fn start_offload(app: AppHandle, state: State<AppState>, job: Job) -> Result<(), String> {
    let mut guard = state.cancel.lock().unwrap();
    if guard.is_some() {
        return Err("Un déchargement est déjà en cours".into());
    }
    let flag = Arc::new(AtomicBool::new(false));
    *guard = Some(flag.clone());
    drop(guard);

    std::thread::spawn(move || {
        let app2 = app.clone();
        let res = run(&job, &flag, &mut |e: Event| {
            let _ = app2.emit("offload", &e);
        });
        if let Err(msg) = res {
            let _ = app.emit("offload", &Event::Log { level: "fatal".into(), message: msg });
        }
        if let Some(st) = tauri::Manager::try_state::<AppState>(&app) {
            *st.cancel.lock().unwrap() = None;
        }
    });
    Ok(())
}

#[tauri::command]
fn cancel_offload(state: State<AppState>) {
    if let Some(f) = state.cancel.lock().unwrap().as_ref() {
        f.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
async fn reverify(path: String) -> Result<(usize, Vec<String>), String> {
    tauri::async_runtime::spawn_blocking(move || verify_mhl(&PathBuf::from(path), &AtomicBool::new(false)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn open_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener().open_path(path, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal_path(app: AppHandle, path: String) -> Result<(), String> {
    app.opener().reveal_item_in_dir(path).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            pick_folder,
            pick_mhl,
            scan_source,
            disk_info,
            start_offload,
            cancel_offload,
            reverify,
            open_path,
            reveal_path
        ])
        .run(tauri::generate_context!())
        .expect("Bandua failed to start");
}
