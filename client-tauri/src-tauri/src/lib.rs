use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::Mutex,
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(target_os = "windows")]
use std::process::Command;
use tauri::{
    AppHandle, Manager, RunEvent, State, WebviewWindow, WindowEvent,
};
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use url::Url;

const DEFAULT_SERVER_BASE_URL: &str = "http://10.10.20.2:6996";
const STATE_FILE_NAME: &str = "client-state.json";
const EXAM_WINDOW_LABEL: &str = "main";
const END_SESSION_EVAL_DELAY_MS: u64 = 260;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClientState {
    server_base_url: String,
    device_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LauncherStateResponse {
    server_base_url: String,
    device_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartExamResponse {
    ok: bool,
    server_base_url: String,
}

#[derive(Debug)]
struct RuntimeState {
    client_state: Mutex<ClientState>,
    allow_exam_close: Mutex<bool>,
}

fn normalize_base_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("URL server wajib diisi.".into());
    }

    let with_protocol = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        let provisional = format!("http://{trimmed}");
        let parsed = Url::parse(&provisional)
            .map_err(|_| "Format URL server tidak valid. Contoh: http://10.10.20.2:6996")?;

        let host = parsed.host_str().unwrap_or_default().to_ascii_lowercase();

        let should_use_http = host == "localhost"
            || host.ends_with(".local")
            || host.parse::<std::net::Ipv4Addr>().map_or(false, |ip| {
                let [a, b, _, _] = ip.octets();
                a == 10
                    || a == 127
                    || (a == 172 && (16..=31).contains(&b))
                    || (a == 192 && b == 168)
                    || (a == 169 && b == 254)
            });

        if should_use_http {
            format!("http://{trimmed}")
        } else {
            format!("https://{trimmed}")
        }
    };

    let parsed = Url::parse(&with_protocol)
        .map_err(|_| "Format URL server tidak valid. Contoh: http://10.10.20.2:6996")?;

    Ok(format!(
        "{}://{}",
        parsed.scheme(),
        parsed.host_str().unwrap_or_default()
    ) + &parsed
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default())
}

fn generate_device_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    format!("desktop-{millis:x}")
}

fn state_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Gagal mendapatkan app data dir: {e}"))?;

    fs::create_dir_all(&dir).map_err(|e| format!("Gagal membuat app data dir: {e}"))?;

    Ok(dir.join(STATE_FILE_NAME))
}

fn load_client_state(app: &AppHandle) -> ClientState {
    let path = match state_file_path(app) {
        Ok(path) => path,
        Err(_) => {
            return ClientState {
                server_base_url: DEFAULT_SERVER_BASE_URL.to_string(),
                device_id: generate_device_id(),
            }
        }
    };

    let fallback = ClientState {
        server_base_url: DEFAULT_SERVER_BASE_URL.to_string(),
        device_id: generate_device_id(),
    };

    let from_file = fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<ClientState>(&raw).ok());

    from_file.unwrap_or(fallback)
}

fn persist_client_state(app: &AppHandle, state: &ClientState) -> Result<(), String> {
    let path = state_file_path(app)?;
    let raw =
        serde_json::to_string_pretty(state).map_err(|e| format!("Gagal serialize state: {e}"))?;

    fs::write(path, raw).map_err(|e| format!("Gagal simpan state: {e}"))
}

fn current_state_payload(state: &RuntimeState) -> LauncherStateResponse {
    let guard = state.client_state.lock().expect("state lock poisoned");

    LauncherStateResponse {
        server_base_url: guard.server_base_url.clone(),
        device_id: guard.device_id.clone(),
    }
}

fn ensure_exam_window_kiosk(window: &WebviewWindow) {
    if !matches!(window.is_fullscreen(), Ok(true)) {
        let _ = window.set_fullscreen(true);
    }
    let _ = window.set_content_protected(false);
    let _ = window.set_skip_taskbar(false);
    let _ = window.set_decorations(false);
    let _ = window.set_resizable(false);
    let _ = window.set_closable(false);
    let _ = window.set_minimizable(false);
    let _ = window.set_maximizable(false);
    let _ = window.set_always_on_top(true);
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

fn exam_window_is_locked(state: &RuntimeState) -> bool {
    state
        .allow_exam_close
        .lock()
        .map(|allow| !*allow)
        .unwrap_or(true)
}

fn exam_kiosk_is_active(app: &AppHandle, state: &RuntimeState) -> bool {
    exam_window_is_locked(state) && app.get_webview_window(EXAM_WINDOW_LABEL).is_some()
}

fn force_exit_now(app: &AppHandle) {
    if let Some(state) = app.try_state::<RuntimeState>() {
        if let Ok(mut allow_close) = state.allow_exam_close.lock() {
            *allow_close = true;
        }
    }

    apply_macos_exam_presentation_lock(app, false);
    app.exit(0);
}

fn write_runtime_log(app: &AppHandle, message: &str) {
    let Ok(mut path) = state_file_path(app) else {
        return;
    };

    path.set_file_name("runtime.log");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|dur| dur.as_millis())
        .unwrap_or(0);

    let line = format!("[{now}] {message}\n");

    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

fn apply_macos_exam_presentation_lock(_app: &AppHandle, _lock: bool) {}

fn activate_exam_kiosk(window: &WebviewWindow) {
    if let Some(state) = window.try_state::<RuntimeState>() {
        if let Ok(mut allow_close) = state.allow_exam_close.lock() {
            *allow_close = false;
        }
    }

    ensure_exam_window_kiosk(window);

    #[cfg(target_os = "windows")]
    kill_windows_explorer();
}

fn restore_launcher_window(window: &WebviewWindow) {
    let _ = window.set_always_on_top(false);
    let _ = window.set_fullscreen(false);
    let _ = window.set_decorations(true);
    let _ = window.set_resizable(false);
    let _ = window.set_closable(true);
    let _ = window.set_minimizable(true);
    let _ = window.set_maximizable(true);
    let _ = window.set_content_protected(false);
    let _ = window.set_skip_taskbar(false);
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.set_title("ExamUQ Client Tauri");

    #[cfg(target_os = "windows")]
    restore_windows_explorer();
}

#[cfg(target_os = "windows")]
fn kill_windows_explorer() {
    let _ = Command::new("taskkill")
        .args(["/F", "/IM", "explorer.exe"])
        .spawn();
}

#[cfg(target_os = "windows")]
fn restore_windows_explorer() {
    let _ = Command::new("explorer.exe").spawn();
}

#[tauri::command]
fn get_launcher_state(state: State<'_, RuntimeState>) -> LauncherStateResponse {
    current_state_payload(&state)
}

#[tauri::command]
fn update_settings(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    server_base_url: String,
) -> Result<LauncherStateResponse, String> {
    let normalized = normalize_base_url(&server_base_url)?;

    {
        let mut guard = state.client_state.lock().map_err(|_| "State lock error")?;
        guard.server_base_url = normalized;
        persist_client_state(&app, &guard)?;
    }

    Ok(current_state_payload(&state))
}

#[tauri::command]
fn open_admin(state: State<'_, RuntimeState>) -> Result<(), String> {
    let guard = state.client_state.lock().map_err(|_| "State lock error")?;
    let admin_url = format!("{}/admin", guard.server_base_url);

    open::that(admin_url).map_err(|e| format!("Gagal membuka admin: {e}"))
}

#[tauri::command]
fn open_exam_direct(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    server_base_url: Option<String>,
) -> Result<(), String> {
    let resolved_base_url = if let Some(raw) = server_base_url {
        normalize_base_url(&raw)?
    } else {
        let guard = state.client_state.lock().map_err(|_| "State lock error")?;
        guard.server_base_url.clone()
    };

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = false;
    }

    let launch_url = format!("{}/", resolved_base_url.trim_end_matches('/'));
    let parsed = Url::parse(&launch_url).map_err(|_| "URL server tidak valid.")?;

    write_runtime_log(&app, &format!("open_exam_direct launch_url={launch_url}"));

    let main_window = app
        .get_webview_window(EXAM_WINDOW_LABEL)
        .ok_or_else(|| "Main window tidak ditemukan.".to_string())?;

    let _ = main_window.navigate(parsed);
    activate_exam_kiosk(&main_window);
    let _ = main_window.set_title("ExamUQ Client - Ujian");
    let _ = main_window.show();
    let _ = main_window.set_focus();

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    Ok(())
}

#[tauri::command]
fn finish_exam_session(app: AppHandle, state: State<'_, RuntimeState>) -> Result<(), String> {
    let main_window = app
        .get_webview_window(EXAM_WINDOW_LABEL)
        .ok_or_else(|| "Main window tidak ditemukan.".to_string())?;

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = true;
    }

    restore_launcher_window(&main_window);
    apply_macos_exam_presentation_lock(&app, false);

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    write_runtime_log(&app, "finish_exam_session invoked");
    Ok(())
}

async fn close_app_with_session_end(
    app: &AppHandle,
    state: &State<'_, RuntimeState>,
) -> Result<(), String> {
    let exam_window = app.get_webview_window(EXAM_WINDOW_LABEL);

    let end_script = r#"
      (async () => {
        const cfg = document.getElementById('playerConfig');
        const sessionId = cfg?.dataset?.sessionId;
        if (!sessionId) return;

        try {
          await fetch('/api/v1/sessions/' + sessionId + '/end', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Accept': 'application/json'
            },
            body: JSON.stringify({ reason: 'client_secret_close' })
          });
        } catch (_e) {
        }
      })();
    "#;

    if let Some(window) = &exam_window {
        let _ = window.eval(end_script);
    }

    thread::sleep(Duration::from_millis(END_SESSION_EVAL_DELAY_MS));

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = true;
    }

    if let Some(window) = &exam_window {
        let _ = window.set_closable(true);
        let _ = window.set_minimizable(true);
        let _ = window.set_maximizable(true);
        let _ = window.set_always_on_top(false);
        let _ = window.set_content_protected(false);
        let _ = window.set_skip_taskbar(false);
        let _ = window.set_fullscreen(false);
        let _ = window.set_decorations(true);
    }

    #[cfg(target_os = "windows")]
    restore_windows_explorer();

    apply_macos_exam_presentation_lock(app, false);

    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn execute_secret_exit(app: AppHandle, state: State<'_, RuntimeState>) -> Result<(), String> {
    close_app_with_session_end(&app, &state).await
}

#[tauri::command]
async fn start_exam(
    app: AppHandle,
    state: State<'_, RuntimeState>,
) -> Result<StartExamResponse, String> {
    let server_base_url = {
        let guard = state.client_state.lock().map_err(|_| "State lock error")?;
        guard.server_base_url.clone()
    };

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = true;
    }

    let launch_url = format!("{}/", server_base_url.trim_end_matches('/'));
    let _ = Url::parse(&launch_url).map_err(|_| "URL server tidak valid.")?;
    write_runtime_log(&app, &format!("start_exam launch_url={launch_url}"));

    let main_window = app
        .get_webview_window(EXAM_WINDOW_LABEL)
        .ok_or_else(|| "Main window tidak ditemukan.".to_string())?;

    activate_exam_kiosk(&main_window);
    let _ = main_window.set_title("ExamUQ Client - Ujian");
    let _ = main_window.show();
    let _ = main_window.set_focus();

    apply_macos_exam_presentation_lock(&app, true);
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    Ok(StartExamResponse {
        ok: true,
        server_base_url,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());

    let global_shortcut_plugin_builder = tauri_plugin_global_shortcut::Builder::new()
        .with_shortcut(Shortcut::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyX,
        ))
        .expect("failed to register ctrl+shift+x global shortcut");

    #[cfg(target_os = "macos")]
    let global_shortcut_plugin_builder = global_shortcut_plugin_builder
        .with_shortcut(Shortcut::new(
            Some(Modifiers::SUPER | Modifiers::SHIFT),
            Code::KeyX,
        ))
        .expect("failed to register cmd+shift+x global shortcut");

    let global_shortcut_plugin = global_shortcut_plugin_builder
        .with_handler(|app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }

            if let Some(state) = app.try_state::<RuntimeState>() {
                if exam_kiosk_is_active(app, &state) {
                    force_exit_now(app);
                }
            }
        })
        .build();

    builder = builder.plugin(global_shortcut_plugin);

    #[cfg(target_os = "macos")]
    {
        builder = builder.enable_macos_default_menu(false);
    }

    let app = builder
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                let _ = app.set_activation_policy(ActivationPolicy::Regular);
                let _ = app.set_dock_visibility(true);
            }

            let mut loaded = load_client_state(&app.handle());

            loaded.server_base_url = normalize_base_url(&loaded.server_base_url)
                .unwrap_or_else(|_| DEFAULT_SERVER_BASE_URL.to_string());

            if loaded.device_id.trim().is_empty() {
                loaded.device_id = generate_device_id();
            }

            persist_client_state(&app.handle(), &loaded)?;

            app.manage(RuntimeState {
                client_state: Mutex::new(loaded),
                allow_exam_close: Mutex::new(true),
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != EXAM_WINDOW_LABEL {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                if let Some(state) = window.try_state::<RuntimeState>() {
                    if let Ok(allow) = state.allow_exam_close.lock() {
                        if !*allow {
                            api.prevent_close();
                        }
                    }
                }
            }

            if let WindowEvent::Focused(false) = event {
                if let Some(state) = window.try_state::<RuntimeState>() {
                    if exam_kiosk_is_active(&window.app_handle(), &state) {
                        if let Some(exam_window) =
                            window.app_handle().get_webview_window(EXAM_WINDOW_LABEL)
                        {
                            ensure_exam_window_kiosk(&exam_window);
                            apply_macos_exam_presentation_lock(&window.app_handle(), true);
                            let _ = exam_window.set_focus();
                        }
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_launcher_state,
            update_settings,
            open_admin,
            start_exam,
            open_exam_direct,
            finish_exam_session,
            execute_secret_exit,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle: &AppHandle, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            let Some(state) = app_handle.try_state::<RuntimeState>() else {
                return;
            };

            if !exam_kiosk_is_active(app_handle, &state) {
                return;
            }

            if let Some(exam_window) = app_handle.get_webview_window(EXAM_WINDOW_LABEL) {
                api.prevent_exit();
                ensure_exam_window_kiosk(&exam_window);
                apply_macos_exam_presentation_lock(app_handle, true);
            }
        }
    });
}
