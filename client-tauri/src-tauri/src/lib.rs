use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    thread,
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{
    AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};
use url::Url;

const DEFAULT_SERVER_BASE_URL: &str = "http://10.10.20.2:6996";
const STATE_FILE_NAME: &str = "client-state.json";
const EXAM_WINDOW_LABEL: &str = "exam";
const SECRET_EXIT_WINDOW_MS: u64 = 5000;
const END_SESSION_EVAL_DELAY_MS: u64 = 260;

const EXAM_WINDOW_INIT_SCRIPT: &str = r#"
(() => {
  const core = window.__TAURI__?.core;

  const invokeSafe = (command) => {
    if (!core?.invoke) return;
    core.invoke(command).catch(() => {});
  };

  window.open = () => null;

  window.addEventListener('keydown', (event) => {
    const ctrlOrMeta = event.ctrlKey || event.metaKey;
    const key = (event.key || '').toLowerCase();

    if (ctrlOrMeta && ['t', 'n', 'w', 'r'].includes(key)) {
      event.preventDefault();
      event.stopImmediatePropagation();
      return;
    }

    if (!ctrlOrMeta || !event.shiftKey) {
      return;
    }

    if (key === 'c') {
      event.preventDefault();
      event.stopImmediatePropagation();
      invokeSafe('arm_secret_exit');
      return;
    }

    if (key === 'b') {
      event.preventDefault();
      event.stopImmediatePropagation();
      invokeSafe('execute_secret_exit');
    }
  }, true);

  document.addEventListener('click', (event) => {
    const target = event.target;
    if (!(target instanceof Element)) return;

    const anchor = target.closest('a[target="_blank"]');
    if (!anchor) return;

    event.preventDefault();
    event.stopImmediatePropagation();
  }, true);
})();
"#;

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
    secret_armed_at: Mutex<Option<Instant>>,
}

fn normalize_base_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("URL server wajib diisi.".into());
    }

    let with_protocol = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{trimmed}")
    };

    let parsed = Url::parse(&with_protocol)
        .map_err(|_| "Format URL server tidak valid. Contoh: http://10.10.20.2:6996")?;

    Ok(format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap_or_default())
        + &parsed
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
    let raw = serde_json::to_string_pretty(state)
        .map_err(|e| format!("Gagal serialize state: {e}"))?;

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
    let _ = window.set_fullscreen(true);
    let _ = window.set_decorations(false);
    let _ = window.set_resizable(false);
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    let _ = window.set_focus();
}

fn hide_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.hide();
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
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
fn arm_secret_exit(state: State<'_, RuntimeState>) {
    if let Ok(mut guard) = state.secret_armed_at.lock() {
        *guard = Some(Instant::now());
    }
}

async fn close_exam_window_with_session_end(
    app: &AppHandle,
    state: &State<'_, RuntimeState>,
) -> Result<(), String> {
    let Some(exam_window) = app.get_webview_window(EXAM_WINDOW_LABEL) else {
        show_main_window(app);
        return Ok(());
    };

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

    let _ = exam_window.eval(end_script);
    thread::sleep(Duration::from_millis(END_SESSION_EVAL_DELAY_MS));

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = true;
    }

    let _ = exam_window.set_always_on_top(false);
    let _ = exam_window.set_fullscreen(false);
    let _ = exam_window.set_decorations(true);
    let _ = exam_window.close();

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = false;
    }

    show_main_window(app);
    Ok(())
}

#[tauri::command]
async fn execute_secret_exit(
    app: AppHandle,
    state: State<'_, RuntimeState>,
) -> Result<(), String> {
    let armed = {
        let mut guard = state
            .secret_armed_at
            .lock()
            .map_err(|_| "State lock error")?;

        let ok = guard
            .map(|instant| instant.elapsed() <= Duration::from_millis(SECRET_EXIT_WINDOW_MS))
            .unwrap_or(false);

        *guard = None;
        ok
    };

    if !armed {
        return Ok(());
    }

    close_exam_window_with_session_end(&app, &state).await
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

    let launch_url = format!("{}/", server_base_url.trim_end_matches('/'));
    let parsed = Url::parse(&launch_url).map_err(|_| "URL server tidak valid.")?;

    if let Some(existing) = app.get_webview_window(EXAM_WINDOW_LABEL) {
        let _ = existing.navigate(parsed.clone());
        ensure_exam_window_kiosk(&existing);
        hide_main_window(&app);

        return Ok(StartExamResponse {
            ok: true,
            server_base_url,
        });
    }

    let exam_window = WebviewWindowBuilder::new(
        &app,
        EXAM_WINDOW_LABEL,
        WebviewUrl::External(parsed),
    )
    .title("ExamUQ Client")
    .decorations(false)
    .resizable(false)
    .fullscreen(true)
    .always_on_top(true)
    .focused(true)
    .user_agent("examuq-client")
    .initialization_script(EXAM_WINDOW_INIT_SCRIPT)
    .build()
    .map_err(|e| format!("Gagal membuka window ujian: {e}"))?;

    ensure_exam_window_kiosk(&exam_window);
    hide_main_window(&app);

    Ok(StartExamResponse {
        ok: true,
        server_base_url,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut loaded = load_client_state(&app.handle());

            loaded.server_base_url = normalize_base_url(&loaded.server_base_url)
                .unwrap_or_else(|_| DEFAULT_SERVER_BASE_URL.to_string());

            if loaded.device_id.trim().is_empty() {
                loaded.device_id = generate_device_id();
            }

            persist_client_state(&app.handle(), &loaded)?;

            app.manage(RuntimeState {
                client_state: Mutex::new(loaded),
                allow_exam_close: Mutex::new(false),
                secret_armed_at: Mutex::new(None),
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
                let _ = window.set_always_on_top(true);
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_launcher_state,
            update_settings,
            open_admin,
            start_exam,
            arm_secret_exit,
            execute_secret_exit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
