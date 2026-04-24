use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::process::Command;
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(target_os = "macos")]
use tauri::ActivationPolicy;
use tauri::{
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder}, AppHandle, Manager,
    LogicalPosition, LogicalSize, PhysicalSize, Position, RunEvent, Size,
    State, WebviewUrl, WebviewWindow, Window, WindowEvent,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_updater::UpdaterExt;
use url::Url;

const DEFAULT_SERVER_BASE_URL: &str = "http://10.10.20.2:6997";
const LEGACY_SERVER_BASE_URL: &str = "http://10.10.20.2:6996";
const LEGACY_SERVER_HOSTS: [&str; 1] = ["kreasai.com"];
const STATE_FILE_NAME: &str = "client-state.json";
const LAUNCHER_WINDOW_LABEL: &str = "main";
const EXAM_WEBVIEW_LABEL: &str = "exam-content";
const EXAM_TOP_BAR_HEIGHT: i32 = 92;
const LAUNCHER_WINDOW_WIDTH: u32 = 460;
const LAUNCHER_WINDOW_HEIGHT: u32 = 600;
const RETURN_TO_LAUNCHER_HOST: &str = "return.examuq.invalid";
const RETURN_TO_LAUNCHER_PATH: &str = "/launcher";
const END_SESSION_EVAL_DELAY_MS: u64 = 260;
#[cfg(target_os = "android")]
const ANDROID_EXAM_LOCK_FILE: &str = "exam-kiosk.lock";
#[cfg(target_os = "android")]
const ANDROID_PERMISSION_GATE_FILE: &str = "permissions-ready.lock";
const DEFAULT_UPDATE_CHANNEL: &str = "stable";
const DEFAULT_UPDATE_ENDPOINT_TEMPLATE: &str =
    "https://updates.examuq.id/{{target}}/{{arch}}/{{current_version}}?channel={channel}";
const BUILD_TIME_UPDATE_CHANNEL: Option<&str> = option_env!("EXAMUQ_UPDATE_CHANNEL");
const BUILD_TIME_ALLOW_BETA: Option<&str> = option_env!("EXAMUQ_ALLOW_BETA");
const BUILD_TIME_UPDATE_ENDPOINT_TEMPLATE: Option<&str> =
    option_env!("EXAMUQ_UPDATER_ENDPOINT_TEMPLATE");

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
    app_version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartExamResponse {
    ok: bool,
    server_base_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct LaunchRequestResponse {
    redirect_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StartExamPayload {
    display_name: String,
    class_room: String,
    token_global: String,
}

#[derive(Debug)]
struct RuntimeState {
    client_state: Mutex<ClientState>,
    allow_exam_close: Mutex<bool>,
    overlay_state: Mutex<Option<ExamOverlayState>>,
    heartbeat_stop_signal: Mutex<Option<Arc<AtomicBool>>>,
    heartbeat_session_key: Mutex<Option<String>>,
    kiosk_watchdog_stop_signal: Mutex<Option<Arc<AtomicBool>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExamOverlayState {
    session_id: String,
    display_name: String,
    deadline_at: String,
    return_url: String,
    api_base: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdaterChannelInfo {
    channel: String,
    endpoint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InstallUpdateResult {
    updated: bool,
    version: Option<String>,
    channel: String,
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
            .map_err(|_| "Format URL server tidak valid. Contoh: http://10.10.20.2:6997")?;

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
        .map_err(|_| "Format URL server tidak valid. Contoh: http://10.10.20.2:6997")?;

    Ok(format!(
        "{}://{}",
        parsed.scheme(),
        parsed.host_str().unwrap_or_default()
    ) + &parsed
        .port()
        .map(|port| format!(":{port}"))
        .unwrap_or_default())
}

fn should_migrate_to_default_server_url(base_url: &str) -> bool {
    if base_url == LEGACY_SERVER_BASE_URL {
        return true;
    }

    if let Ok(parsed) = Url::parse(base_url) {
        if let Some(host) = parsed.host_str() {
            let host = host.to_ascii_lowercase();
            return LEGACY_SERVER_HOSTS.iter().any(|legacy_host| {
                host == *legacy_host || host.ends_with(&format!(".{legacy_host}"))
            });
        }
    }

    false
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
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

fn selected_update_channel() -> String {
    let allow_beta = BUILD_TIME_ALLOW_BETA
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false);

    let raw = BUILD_TIME_UPDATE_CHANNEL
        .unwrap_or(DEFAULT_UPDATE_CHANNEL)
        .to_string();
    let normalized = raw.trim().to_ascii_lowercase();

    if normalized == "beta" && !allow_beta {
        return DEFAULT_UPDATE_CHANNEL.to_string();
    }

    match normalized.as_str() {
        "stable" | "beta" => normalized,
        _ => DEFAULT_UPDATE_CHANNEL.to_string(),
    }
}

fn updater_endpoint_for_channel(channel: &str) -> String {
    let template = BUILD_TIME_UPDATE_ENDPOINT_TEMPLATE
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_UPDATE_ENDPOINT_TEMPLATE.to_string());

    template.replace("{channel}", channel)
}

fn validate_updater_endpoint(endpoint: &Url) -> Result<(), String> {
    if endpoint.scheme() != "https" {
        return Err("Updater endpoint wajib menggunakan HTTPS.".to_string());
    }

    if endpoint.host_str().unwrap_or_default().is_empty() {
        return Err("Updater endpoint wajib memiliki host yang valid.".to_string());
    }

    Ok(())
}

#[tauri::command]
fn get_updater_channel_info() -> UpdaterChannelInfo {
    let channel = selected_update_channel();
    let endpoint = updater_endpoint_for_channel(&channel);

    UpdaterChannelInfo { channel, endpoint }
}

#[tauri::command]
async fn check_for_updates(app: AppHandle) -> Result<Option<String>, String> {
    let channel = selected_update_channel();
    let endpoint = updater_endpoint_for_channel(&channel);

    let update_url = Url::parse(&endpoint)
        .map_err(|e| format!("Updater endpoint tidak valid untuk channel {channel}: {e}"))?;
    validate_updater_endpoint(&update_url)?;

    let update = app
        .updater_builder()
        .endpoints(vec![update_url])
        .map_err(|e| format!("Gagal set updater endpoint: {e}"))?
        .build()
        .map_err(|e| format!("Gagal build updater: {e}"))?
        .check()
        .await
        .map_err(|e| format!("Gagal check update: {e}"))?;

    Ok(update.map(|item| item.version))
}

#[tauri::command]
async fn install_available_update(app: AppHandle) -> Result<InstallUpdateResult, String> {
    let channel = selected_update_channel();
    let endpoint = updater_endpoint_for_channel(&channel);

    let update_url = Url::parse(&endpoint)
        .map_err(|e| format!("Updater endpoint tidak valid untuk channel {channel}: {e}"))?;
    validate_updater_endpoint(&update_url)?;

    let updater = app
        .updater_builder()
        .endpoints(vec![update_url])
        .map_err(|e| format!("Gagal set updater endpoint: {e}"))?
        .build()
        .map_err(|e| format!("Gagal build updater: {e}"))?;

    let update = updater
        .check()
        .await
        .map_err(|e| format!("Gagal check update: {e}"))?;

    let Some(update) = update else {
        return Ok(InstallUpdateResult {
            updated: false,
            version: None,
            channel,
        });
    };

    let target_version = update.version.clone();

    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| format!("Gagal download/install update: {e}"))?;

    app.request_restart();

    Ok(InstallUpdateResult {
        updated: true,
        version: Some(target_version),
        channel,
    })
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn ensure_exam_window_kiosk(window: &WebviewWindow) {
    if matches!(window.is_minimized(), Ok(true)) {
        let _ = window.unminimize();
    }
    let _ = window.set_content_protected(true);
    let _ = window.set_skip_taskbar(true);
    let _ = window.set_always_on_top(true);
    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn ensure_exam_window_kiosk(_window: &WebviewWindow) {}

fn exam_window_is_locked(state: &RuntimeState) -> bool {
    state
        .allow_exam_close
        .lock()
        .map(|allow| !*allow)
        .unwrap_or(true)
}

fn exam_kiosk_is_active(app: &AppHandle, state: &RuntimeState) -> bool {
    let _ = app;
    exam_window_is_locked(state)
}

fn launcher_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    app.get_webview_window(LAUNCHER_WINDOW_LABEL)
        .ok_or_else(|| "Main window tidak ditemukan.".to_string())
}

fn launcher_host_window(app: &AppHandle) -> Result<Window, String> {
    app.get_window(LAUNCHER_WINDOW_LABEL)
        .ok_or_else(|| "Host window tidak ditemukan.".to_string())
}

fn is_return_to_launcher_url(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some(RETURN_TO_LAUNCHER_HOST)
        && url.path() == RETURN_TO_LAUNCHER_PATH
}

fn return_message_from_url(url: &Url) -> String {
    url.query_pairs()
        .find_map(|(key, value)| (key == "message").then(|| value.into_owned()))
        .filter(|message| !message.trim().is_empty())
        .unwrap_or_else(|| "Mode ujian dihentikan dan kembali ke halaman awal.".to_string())
}

fn build_exam_launch_url(server_base_url: &str, device_id: &str) -> Result<Url, String> {
    let mut launch_url = Url::parse(server_base_url)
        .map_err(|_| "URL server tidak valid.".to_string())?;

    launch_url.set_path("/");
    launch_url.set_query(None);
    launch_url.set_fragment(None);

    launch_url
        .query_pairs_mut()
        .append_pair("source", "client")
        .append_pair("client_type", "desktop_client")
        .append_pair("device_id", device_id);

    Ok(launch_url)
}

fn build_launch_request_url(server_base_url: &str) -> Result<Url, String> {
    Url::parse(&format!(
        "{}/api/v1/launch/request",
        server_base_url.trim_end_matches('/')
    ))
    .map_err(|_| "URL launch API tidak valid.".to_string())
}

fn request_launch_from_backend(
    server_base_url: &str,
    device_id: &str,
    payload: &StartExamPayload,
) -> Result<Url, String> {
    let api_url = build_launch_request_url(server_base_url)?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Gagal membuat HTTP client launch: {e}"))?;

    let response = client
        .post(api_url)
        .header("X-ExamUQ-Client-Type", "desktop_client")
        .header("X-ExamUQ-Source", "client")
        .header("X-ExamUQ-Device-Id", device_id)
        .json(&serde_json::json!({
            "display_name": payload.display_name,
            "class_room": payload.class_room,
            "token_global": payload.token_global,
            "client_type": "desktop_client",
            "device_id": device_id,
        }))
        .send()
        .map_err(|e| format!("Gagal mengirim data peserta/token: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().unwrap_or_default();
        let body_message = serde_json::from_str::<serde_json::Value>(&body_text)
            .ok()
            .and_then(|body| body.get("message").and_then(|m| m.as_str()).map(|m| m.to_string()));

        let message = match status.as_u16() {
            403 | 404 | 405 => format!(
                "Backend belum mendukung form client ExamUQ (status {}). Update backend terbaru lalu jalankan php artisan optimize:clear, route:clear, config:clear.",
                status
            ),
            422 => body_message.unwrap_or_else(|| {
                "Token tidak valid atau ujian tidak aktif di backend.".to_string()
            }),
            500..=599 => format!(
                "Backend error (status {}). Cek log server dan pastikan route launch request aktif.",
                status
            ),
            _ => body_message.unwrap_or_else(|| {
                if body_text.trim().is_empty() {
                    format!("Launch ditolak server (status {status})")
                } else {
                    format!("Launch ditolak server (status {}): {}", status, body_text.trim())
                }
            }),
        };

        return Err(message);
    }

    let body: LaunchRequestResponse = response
        .json()
        .map_err(|e| format!("Respons launch tidak valid: {e}"))?;

    Url::parse(&body.redirect_url).map_err(|_| "URL redirect ujian tidak valid dari backend.".to_string())
}

fn enter_exam_mode_runtime(app: &AppHandle, state: &RuntimeState) -> Result<(), String> {
    if let Some(existing_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
        let _ = existing_webview.close();
    }

    stop_exam_session_heartbeat(state);

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = false;
    }

    {
        let mut overlay = state.overlay_state.lock().map_err(|_| "State lock error")?;
        *overlay = Some(default_exam_overlay_state());
    }

    let launcher = launcher_window(app)?;
    let _ = launcher.set_title("ExamUQ Client - Ujian");
    let _ = launcher.show();
    let _ = launcher.set_decorations(false);
    let _ = launcher.set_resizable(false);
    let _ = launcher.set_fullscreen(true);
    let _ = launcher.set_always_on_top(true);
    let _ = launcher.set_closable(false);
    let _ = launcher.set_maximizable(false);
    let _ = launcher.set_minimizable(false);
    activate_exam_kiosk(&launcher);
    start_kiosk_watchdog(app, state);

    enter_exam_mode_ui(app);
    sync_exam_state_to_launcher(app, &default_exam_overlay_state());
    sync_exam_loading_state(app, false, "Isi data peserta untuk memulai ujian.");

    apply_macos_exam_presentation_lock(app, true);

    Ok(())
}

fn parse_overlay_state_from_url(url: &Url) -> Option<ExamOverlayState> {
    let mut session_id = String::new();
    let mut display_name = String::new();
    let mut deadline_at = String::new();
    let mut return_url = String::new();
    let mut api_base = String::new();

    let mut apply_pair = |key: &str, value: String| match key {
        "examuq_session_id" => session_id = value,
        "examuq_display_name" => display_name = value,
        "examuq_deadline_at" => deadline_at = value,
        "examuq_return_url" => return_url = value,
        "examuq_api_base" => api_base = value,
        _ => {}
    };

    if let Some(query) = url.query() {
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            apply_pair(key.as_ref(), value.into_owned());
        }
    }

    if let Some(fragment) = url.fragment() {
        for (key, value) in url::form_urlencoded::parse(fragment.as_bytes()) {
            apply_pair(key.as_ref(), value.into_owned());
        }
    }

    if session_id.trim().is_empty() || return_url.trim().is_empty() {
        return None;
    }

    Some(ExamOverlayState {
        session_id,
        display_name,
        deadline_at,
        return_url,
        api_base,
    })
}

fn default_exam_overlay_state() -> ExamOverlayState {
    ExamOverlayState {
        session_id: String::new(),
        display_name: "Peserta".to_string(),
        deadline_at: String::new(),
        return_url: format!("https://{RETURN_TO_LAUNCHER_HOST}{RETURN_TO_LAUNCHER_PATH}"),
        api_base: String::new(),
    }
}

fn merge_exam_overlay_state(existing: Option<ExamOverlayState>, incoming: ExamOverlayState) -> ExamOverlayState {
    let mut merged = existing.unwrap_or_else(default_exam_overlay_state);

    if !incoming.session_id.trim().is_empty() {
        merged.session_id = incoming.session_id;
    }
    if !incoming.display_name.trim().is_empty() {
        merged.display_name = incoming.display_name;
    }
    if !incoming.deadline_at.trim().is_empty() {
        merged.deadline_at = incoming.deadline_at;
    }
    if !incoming.return_url.trim().is_empty() {
        merged.return_url = incoming.return_url;
    }
    if !incoming.api_base.trim().is_empty() {
        merged.api_base = incoming.api_base;
    }

    merged
}

fn sync_exam_loading_state(app: &AppHandle, active: bool, message: &str) {
    let payload_active = if active { "true" } else { "false" };
    let payload_message = serde_json::to_string(message).unwrap_or_else(|_| "\"\"".to_string());
    if let Ok(launcher) = launcher_window(app) {
        let _ = launcher.eval(&format!(
            "window.__EXAMUQ_SET_EXAM_LOADING__ && window.__EXAMUQ_SET_EXAM_LOADING__({payload_active}, {payload_message});"
        ));
    }
}

fn sync_exam_state_to_launcher(app: &AppHandle, overlay: &ExamOverlayState) {
    let payload = serde_json::to_string(overlay).unwrap_or_else(|_| "{}".to_string());
    if let Ok(launcher) = launcher_window(app) {
        let _ = launcher.eval(&format!(
            "window.__EXAMUQ_SYNC_EXAM_STATE__ && window.__EXAMUQ_SYNC_EXAM_STATE__({payload});"
        ));
    }
}

fn enter_exam_mode_ui(app: &AppHandle) {
    if let Ok(launcher) = launcher_window(app) {
        let _ = launcher.eval("window.__EXAMUQ_ENTER_EXAM_MODE__ && window.__EXAMUQ_ENTER_EXAM_MODE__();");
    }
}

fn store_exam_overlay_state(
    app: &AppHandle,
    state: &RuntimeState,
    overlay: ExamOverlayState,
) -> Result<ExamOverlayState, String> {
    let merged = {
        let mut guard = state.overlay_state.lock().map_err(|_| "State lock error")?;
        let merged = merge_exam_overlay_state(guard.clone(), overlay);
        *guard = Some(merged.clone());
        merged
    };

    sync_exam_state_to_launcher(app, &merged);
    Ok(merged)
}

fn update_overlay_state_from_url(app: &AppHandle, state: &RuntimeState, url: &Url) {
    if let Some(parsed) = parse_overlay_state_from_url(url) {
        if let Ok(overlay) = store_exam_overlay_state(app, state, parsed) {
            ensure_exam_session_heartbeat(app, state, &overlay);
        }
    }
}

fn session_api_url(api_base: &str, session_id: &str, action: &str) -> Result<String, String> {
    let base = api_base.trim();
    let session = session_id.trim();

    if base.is_empty() || session.is_empty() {
        return Err("metadata sesi belum lengkap".to_string());
    }

    Ok(format!(
        "{}/api/v1/sessions/{}/{}",
        base.trim_end_matches('/'),
        session,
        action.trim_start_matches('/')
    ))
}

fn send_session_heartbeat(overlay: &ExamOverlayState) -> Result<(), String> {
    let heartbeat_url = session_api_url(&overlay.api_base, &overlay.session_id, "heartbeat")?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .map_err(|e| format!("gagal membuat HTTP client heartbeat: {e}"))?;

    let response = client
        .post(&heartbeat_url)
        .send()
        .map_err(|e| format!("gagal kirim heartbeat: {e}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    Err(format!(
        "heartbeat ditolak server (status {})",
        response.status()
    ))
}

fn send_session_end(overlay: &ExamOverlayState, reason: &str) -> Result<(), String> {
    let end_url = session_api_url(&overlay.api_base, &overlay.session_id, "end")?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("gagal membuat HTTP client end-session: {e}"))?;

    let response = client
        .post(&end_url)
        .json(&serde_json::json!({ "reason": reason }))
        .send()
        .map_err(|e| format!("gagal mengakhiri sesi: {e}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    Err(format!(
        "end-session ditolak server (status {})",
        response.status()
    ))
}

fn stop_exam_session_heartbeat(state: &RuntimeState) {
    if let Ok(mut guard) = state.heartbeat_stop_signal.lock() {
        if let Some(stop_signal) = guard.take() {
            stop_signal.store(true, Ordering::Relaxed);
        }
    }

    if let Ok(mut key_guard) = state.heartbeat_session_key.lock() {
        *key_guard = None;
    }
}

fn ensure_exam_session_heartbeat(app: &AppHandle, state: &RuntimeState, overlay: &ExamOverlayState) {
    let session_key = format!(
        "{}|{}",
        overlay.api_base.trim().to_ascii_lowercase(),
        overlay.session_id.trim()
    );

    if overlay.api_base.trim().is_empty() || overlay.session_id.trim().is_empty() {
        return;
    }

    if let Ok(existing_key_guard) = state.heartbeat_session_key.lock() {
        if existing_key_guard.as_deref() == Some(session_key.as_str()) {
            return;
        }
    }

    stop_exam_session_heartbeat(state);

    let stop_signal = Arc::new(AtomicBool::new(false));
    let loop_stop = Arc::clone(&stop_signal);
    let loop_overlay = overlay.clone();
    let app_handle = app.clone();

    thread::spawn(move || {
        write_runtime_log(
            &app_handle,
            &format!(
                "heartbeat start session={} api_base={}",
                loop_overlay.session_id, loop_overlay.api_base
            ),
        );

        loop {
            if loop_stop.load(Ordering::Relaxed) {
                break;
            }

            if let Err(error) = send_session_heartbeat(&loop_overlay) {
                write_runtime_log(
                    &app_handle,
                    &format!(
                        "heartbeat warn session={} reason={error}",
                        loop_overlay.session_id
                    ),
                );
            }

            for _ in 0..10 {
                if loop_stop.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(Duration::from_secs(1));
            }
        }

        write_runtime_log(
            &app_handle,
            &format!("heartbeat stop session={}", loop_overlay.session_id),
        );
    });

    if let Ok(mut guard) = state.heartbeat_stop_signal.lock() {
        *guard = Some(stop_signal);
    }
    if let Ok(mut key_guard) = state.heartbeat_session_key.lock() {
        *key_guard = Some(session_key);
    }
}

fn active_overlay_state(state: &RuntimeState) -> Option<ExamOverlayState> {
    state.overlay_state.lock().ok().and_then(|guard| guard.clone())
}

fn end_active_session_if_any(app: &AppHandle, state: &RuntimeState, reason: &str) {
    let Some(overlay) = active_overlay_state(state) else {
        return;
    };

    if overlay.session_id.trim().is_empty() || overlay.api_base.trim().is_empty() {
        return;
    }

    if let Err(error) = send_session_end(&overlay, reason) {
        write_runtime_log(
            app,
            &format!(
                "end-session warn session={} reason={reason} error={error}",
                overlay.session_id
            ),
        );
    } else {
        write_runtime_log(
            app,
            &format!(
                "end-session ok session={} reason={reason}",
                overlay.session_id
            ),
        );
    }
}

fn resize_exam_webview(app: &AppHandle) -> Result<(), String> {
    let Some(webview) = app.get_webview(EXAM_WEBVIEW_LABEL) else {
        return Ok(());
    };

    let host = launcher_host_window(app)?;
    let size = host
        .inner_size()
        .map_err(|e| format!("Gagal membaca ukuran window ujian: {e}"))?;
    let scale_factor = host
        .scale_factor()
        .map_err(|e| format!("Gagal membaca skala layar ujian: {e}"))?;

    let logical_width = f64::from(size.width) / scale_factor;
    let logical_height = f64::from(size.height) / scale_factor;
    let top_bar_height = f64::from(EXAM_TOP_BAR_HEIGHT);
    let webview_height = (logical_height - top_bar_height).max(1.0);

    webview
        .set_position(Position::Logical(LogicalPosition::new(0.0, top_bar_height)))
        .map_err(|e| format!("Gagal mengatur posisi area ujian: {e}"))?;
    webview
        .set_size(Size::Logical(LogicalSize::new(logical_width, webview_height)))
        .map_err(|e| format!("Gagal mengatur ukuran area ujian: {e}"))?;
    let _ = webview.set_auto_resize(false);
    let _ = webview.show();

    Ok(())
}

fn focus_exam_runtime(app: &AppHandle, state: &RuntimeState) {
    if let Ok(launcher) = launcher_window(app) {
        ensure_exam_window_kiosk(&launcher);
        if exam_kiosk_is_active(app, state) {
            apply_macos_exam_presentation_lock(app, true);
        }
    }

    if let Some(webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
        let _ = webview.set_focus();
    }
}

#[tauri::command]
fn get_exam_overlay_state(state: State<'_, RuntimeState>) -> Result<ExamOverlayState, String> {
    let guard = state.overlay_state.lock().map_err(|_| "State lock error")?;
    Ok(guard.clone().unwrap_or_else(default_exam_overlay_state))
}

fn open_exam_window(app: &AppHandle, state: &RuntimeState, launch_url: Url) -> Result<(), String> {
    let already_in_exam_mode = exam_window_is_locked(state);
    if !already_in_exam_mode {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = false;
    }

    let launcher = launcher_window(app)?;
    let host_window = launcher_host_window(app)?;

    if let Some(existing_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
        let _ = existing_webview.close();
    }

    {
        let mut overlay = state.overlay_state.lock().map_err(|_| "State lock error")?;
        *overlay = Some(
            parse_overlay_state_from_url(&launch_url).unwrap_or_else(default_exam_overlay_state),
        );
    }

    if let Some(overlay) = active_overlay_state(state) {
        ensure_exam_session_heartbeat(app, state, &overlay);
    }

    enter_exam_mode_ui(app);
    sync_exam_loading_state(app, true, "Memuat portal ujian...");
    sync_exam_state_to_launcher(
        app,
        &state
            .overlay_state
            .lock()
            .map_err(|_| "State lock error")?
            .clone()
            .unwrap_or_else(default_exam_overlay_state),
    );

    let _ = launcher.set_title("ExamUQ Client - Ujian");
    let _ = launcher.show();
    if !already_in_exam_mode {
        let _ = launcher.set_decorations(false);
        let _ = launcher.set_resizable(false);
        let _ = launcher.set_fullscreen(true);
        let _ = launcher.set_always_on_top(true);
        let _ = launcher.set_closable(false);
        let _ = launcher.set_maximizable(false);
        let _ = launcher.set_minimizable(false);
        activate_exam_kiosk(&launcher);
        start_kiosk_watchdog(app, state);
    } else {
        let _ = launcher.set_always_on_top(true);
        let _ = launcher.set_focus();
    }

    let nav_app = app.clone();
    let load_app = app.clone();
    let popup_app = app.clone();
    let webview_builder = WebviewBuilder::new(EXAM_WEBVIEW_LABEL, WebviewUrl::External(launch_url))
        .on_navigation(move |url| {
            if let Some(state) = nav_app.try_state::<RuntimeState>() {
                update_overlay_state_from_url(&nav_app, &state, url);
            }

            if !is_return_to_launcher_url(url) {
                return true;
            }

            if let Some(state) = nav_app.try_state::<RuntimeState>() {
                let _ = return_to_launcher_runtime(&nav_app, &state, &return_message_from_url(url));
            }

            false
        })
        .on_page_load(move |_webview, payload| {
            if let Some(state) = load_app.try_state::<RuntimeState>() {
                update_overlay_state_from_url(&load_app, &state, payload.url());
            }

            let message = match payload.event() {
                PageLoadEvent::Started => "Memuat halaman ujian...",
                PageLoadEvent::Finished => "Portal ujian siap.",
            };

            sync_exam_loading_state(&load_app, matches!(payload.event(), PageLoadEvent::Started), message);
        })
        .on_new_window(move |url, _features| {
            if is_return_to_launcher_url(&url) {
                if let Some(state) = popup_app.try_state::<RuntimeState>() {
                    let _ = return_to_launcher_runtime(&popup_app, &state, &return_message_from_url(&url));
                }
                return NewWindowResponse::Deny;
            }

            if let Some(webview) = popup_app.get_webview(EXAM_WEBVIEW_LABEL) {
                let _ = webview.navigate(url.clone());
            }
            sync_exam_loading_state(&popup_app, true, "Mengarahkan portal ujian...");
            NewWindowResponse::Deny
        });

    let host_size = host_window
        .inner_size()
        .map_err(|e| format!("Gagal membaca ukuran host ujian: {e}"))?;
    let scale_factor = host_window
        .scale_factor()
        .map_err(|e| format!("Gagal membaca skala host ujian: {e}"))?;
    let logical_width = f64::from(host_size.width) / scale_factor;
    let logical_height = f64::from(host_size.height) / scale_factor;
    let top_bar_height = f64::from(EXAM_TOP_BAR_HEIGHT);
    let webview_height = (logical_height - top_bar_height).max(1.0);

    host_window
        .add_child(
            webview_builder,
            LogicalPosition::new(0.0, top_bar_height),
            LogicalSize::new(logical_width, webview_height),
        )
        .map_err(|e| format!("Gagal membuka area ujian: {e}"))?;

    resize_exam_webview(app)?;
    focus_exam_runtime(app, state);

    apply_macos_exam_presentation_lock(app, true);
    #[cfg(target_os = "macos")]
    {
        let _ = app.set_activation_policy(ActivationPolicy::Regular);
        let _ = app.set_dock_visibility(true);
    }

    Ok(())
}

fn return_to_launcher_runtime(
    app: &AppHandle,
    state: &RuntimeState,
    reason: &str,
) -> Result<(), String> {
    let launcher = launcher_window(app)?;

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = true;
    }

    end_active_session_if_any(app, state, reason);
    stop_exam_session_heartbeat(state);
    stop_kiosk_watchdog(state);

    {
        let mut overlay = state.overlay_state.lock().map_err(|_| "State lock error")?;
        *overlay = None;
    }

    if let Some(exam_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
        let _ = exam_webview.close();
    }

    restore_launcher_window(&launcher);
    apply_macos_exam_presentation_lock(app, false);
    let payload = serde_json::to_string(reason).unwrap_or_else(|_| "\"\"".to_string());
    let _ = launcher.eval(&format!(
        "window.__EXAMUQ_RETURN_TO_LAUNCHER__ && window.__EXAMUQ_RETURN_TO_LAUNCHER__({payload});"
    ));
    write_runtime_log(app, reason);

    Ok(())
}

fn force_return_to_launcher_best_effort(app: &AppHandle, state: &RuntimeState, reason: &str) {
    if let Ok(mut allow_close) = state.allow_exam_close.lock() {
        *allow_close = true;
    }

    end_active_session_if_any(app, state, reason);
    stop_exam_session_heartbeat(state);
    stop_kiosk_watchdog(state);

    if let Ok(mut overlay) = state.overlay_state.lock() {
        *overlay = None;
    }

    if let Some(exam_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
        let _ = exam_webview.close();
    }

    if let Ok(launcher) = launcher_window(app) {
        restore_launcher_window(&launcher);
        let payload = serde_json::to_string(reason).unwrap_or_else(|_| "\"\"".to_string());
        let _ = launcher.eval(&format!(
            "window.__EXAMUQ_RETURN_TO_LAUNCHER__ && window.__EXAMUQ_RETURN_TO_LAUNCHER__({payload});"
        ));
    }

    apply_macos_exam_presentation_lock(app, false);
    write_runtime_log(app, &format!("force-return-to-launcher {reason}"));
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

fn stop_kiosk_watchdog(state: &RuntimeState) {
    if let Ok(mut guard) = state.kiosk_watchdog_stop_signal.lock() {
        if let Some(stop_signal) = guard.take() {
            stop_signal.store(true, Ordering::Relaxed);
        }
    }
}

fn start_kiosk_watchdog(app: &AppHandle, state: &RuntimeState) {
    stop_kiosk_watchdog(state);

    let stop_signal = Arc::new(AtomicBool::new(false));
    let loop_stop = Arc::clone(&stop_signal);
    let app_handle = app.clone();

    thread::spawn(move || {
        write_runtime_log(&app_handle, "kiosk-watchdog start");

        loop {
            if loop_stop.load(Ordering::Relaxed) {
                break;
            }

            let Some(state) = app_handle.try_state::<RuntimeState>() else {
                break;
            };

            if !exam_kiosk_is_active(&app_handle, &state) {
                thread::sleep(Duration::from_millis(300));
                continue;
            }

            if let Ok(launcher) = launcher_window(&app_handle) {
                ensure_exam_window_kiosk(&launcher);
                let _ = launcher.set_always_on_top(true);
                let _ = launcher.show();
                let _ = launcher.set_focus();
            }

            if let Some(exam_webview) = app_handle.get_webview(EXAM_WEBVIEW_LABEL) {
                let _ = exam_webview.set_focus();
            }

            apply_macos_exam_presentation_lock(&app_handle, true);
            thread::sleep(Duration::from_millis(350));
        }

        write_runtime_log(&app_handle, "kiosk-watchdog stop");
    });

    if let Ok(mut guard) = state.kiosk_watchdog_stop_signal.lock() {
        *guard = Some(stop_signal);
    }
}

fn close_app_with_session_end_runtime(app: &AppHandle, state: &RuntimeState) -> Result<(), String> {
    end_active_session_if_any(app, state, "client_secret_close");
    stop_exam_session_heartbeat(state);
    stop_kiosk_watchdog(state);

    let exam_window = app.get_webview(EXAM_WEBVIEW_LABEL);

    thread::sleep(Duration::from_millis(END_SESSION_EVAL_DELAY_MS));

    {
        let mut allow_close = state
            .allow_exam_close
            .lock()
            .map_err(|_| "State lock error")?;
        *allow_close = true;
    }

    if let Some(window) = &exam_window {
        let _ = window.close();
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    if let Some(window) = app.get_webview_window(LAUNCHER_WINDOW_LABEL) {
        let _ = window.set_always_on_top(false);
        let _ = window.set_content_protected(false);
        let _ = window.set_skip_taskbar(false);
        let _ = window.set_fullscreen(false);
    }

    set_android_exam_lock(app, false);

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

#[cfg(target_os = "android")]
fn set_android_exam_lock(app: &AppHandle, locked: bool) {
    let Ok(mut path) = state_file_path(app) else {
        return;
    };

    path.set_file_name(ANDROID_EXAM_LOCK_FILE);
    let value = if locked { "1" } else { "0" };
    let _ = fs::write(path, value);
}

#[cfg(target_os = "android")]
fn android_permissions_are_ready(app: &AppHandle) -> bool {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(mut path) = state_file_path(app) {
        path.set_file_name(ANDROID_PERMISSION_GATE_FILE);
        candidates.push(path);
    }

    if let Ok(path) = app.path().app_data_dir() {
        candidates.push(path.join(ANDROID_PERMISSION_GATE_FILE));
    }

    if let Ok(path) = app.path().app_config_dir() {
        candidates.push(path.join(ANDROID_PERMISSION_GATE_FILE));
    }

    if let Ok(path) = app.path().app_cache_dir() {
        candidates.push(path.join(ANDROID_PERMISSION_GATE_FILE));
    }

    candidates.sort();
    candidates.dedup();

    let mut saw_explicit_false = false;
    let mut saw_marker = false;

    for path in candidates {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };

        saw_marker = true;
        let flag = raw.trim();

        if flag == "1" {
            return true;
        }

        if flag == "0" {
            saw_explicit_false = true;
        }
    }

    if saw_explicit_false {
        return false;
    }

    if !saw_marker {
        write_runtime_log(
            app,
            "android permission gate marker not found; falling back to native gate enforcement",
        );
        return true;
    }

    false
}

#[cfg(not(target_os = "android"))]
fn android_permissions_are_ready(_app: &AppHandle) -> bool {
    true
}

#[cfg(not(target_os = "android"))]
fn set_android_exam_lock(_app: &AppHandle, _locked: bool) {}

fn apply_macos_exam_presentation_lock(_app: &AppHandle, _lock: bool) {}

fn activate_exam_kiosk(window: &WebviewWindow) {
    if let Some(state) = window.try_state::<RuntimeState>() {
        if let Ok(mut allow_close) = state.allow_exam_close.lock() {
            *allow_close = false;
        }
    }

    ensure_exam_window_kiosk(window);
    set_android_exam_lock(&window.app_handle(), true);

    #[cfg(target_os = "windows")]
    kill_windows_explorer();
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn restore_launcher_window(window: &WebviewWindow) {
    let _ = window.set_always_on_top(false);
    let _ = window.set_fullscreen(false);
    let _ = window.set_decorations(true);
    let _ = window.set_resizable(false);
    let _ = window.set_closable(true);
    let _ = window.set_maximizable(false);
    let _ = window.set_minimizable(false);
    let _ = window.set_content_protected(false);
    let _ = window.set_skip_taskbar(false);
    let _ = window.set_size(Size::Physical(PhysicalSize::new(
        LAUNCHER_WINDOW_WIDTH,
        LAUNCHER_WINDOW_HEIGHT,
    )));
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
    let _ = window.set_title("ExamUQ Client");
    set_android_exam_lock(&window.app_handle(), false);

    #[cfg(target_os = "windows")]
    restore_windows_explorer();
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn restore_launcher_window(window: &WebviewWindow) {
    set_android_exam_lock(&window.app_handle(), false);
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
    let mut admin_url = Url::parse(&format!("{}/admin", guard.server_base_url.trim_end_matches('/')))
        .map_err(|_| "URL server tidak valid.")?;

    admin_url
        .query_pairs_mut()
        .append_pair("source", "client")
        .append_pair("client_type", "desktop_client")
        .append_pair("device_id", &guard.device_id);

    open::that(admin_url.to_string()).map_err(|e| format!("Gagal membuka admin: {e}"))
}

#[tauri::command]
async fn open_exam_direct(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    server_base_url: Option<String>,
) -> Result<(), String> {
    if !android_permissions_are_ready(&app) {
        write_runtime_log(
            &app,
            "open_exam_direct: permission marker false; continue and defer enforcement to native gate",
        );
    }

    let (resolved_base_url, device_id) = {
        let guard = state.client_state.lock().map_err(|_| "State lock error")?;
        let base = if let Some(raw) = server_base_url {
            normalize_base_url(&raw)?
        } else {
            guard.server_base_url.clone()
        };

        (base, guard.device_id.clone())
    };

    let parsed = build_exam_launch_url(&resolved_base_url, &device_id)?;
    let launch_url = parsed.to_string();

    write_runtime_log(&app, &format!("open_exam_direct launch_url={launch_url}"));

    open_exam_window(&app, &state, parsed)
}

#[tauri::command]
fn finish_exam_session(app: AppHandle, state: State<'_, RuntimeState>) -> Result<(), String> {
    if let Err(error) = return_to_launcher_runtime(&app, &state, "finish_exam_session invoked") {
        write_runtime_log(
            &app,
            &format!("finish_exam_session fallback_return reason={error}"),
        );
        force_return_to_launcher_best_effort(&app, &state, "finish_exam_session fallback");
    }

    Ok(())
}

async fn close_app_with_session_end(
    app: &AppHandle,
    state: &State<'_, RuntimeState>,
) -> Result<(), String> {
    close_app_with_session_end_runtime(app, state)
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
    if !android_permissions_are_ready(&app) {
        write_runtime_log(
            &app,
            "start_exam: permission marker false; continue and defer enforcement to native gate",
        );
    }

    let server_base_url = {
        let guard = state.client_state.lock().map_err(|_| "State lock error")?;
        guard.server_base_url.clone()
    };

    write_runtime_log(&app, "start_exam enter_exam_mode_runtime");
    enter_exam_mode_runtime(&app, &state)?;

    Ok(StartExamResponse {
        ok: true,
        server_base_url,
    })
}

#[tauri::command]
async fn launch_exam_from_client(
    app: AppHandle,
    state: State<'_, RuntimeState>,
    payload: StartExamPayload,
) -> Result<StartExamResponse, String> {
    if !android_permissions_are_ready(&app) {
        write_runtime_log(
            &app,
            "launch_exam_from_client: permission marker false; continue and defer enforcement to native gate",
        );
    }

    let display_name = payload.display_name.trim().to_string();
    let class_room = payload.class_room.trim().to_string();
    let token_global = payload.token_global.trim().to_string();

    if display_name.is_empty() {
        return Err("Nama peserta wajib diisi.".to_string());
    }
    if class_room.is_empty() {
        return Err("Kelas wajib diisi.".to_string());
    }
    if token_global.is_empty() {
        return Err("Token ujian wajib diisi.".to_string());
    }

    let normalized_payload = StartExamPayload {
        display_name,
        class_room,
        token_global,
    };

    let (server_base_url, device_id) = {
        let guard = state.client_state.lock().map_err(|_| "State lock error")?;
        (guard.server_base_url.clone(), guard.device_id.clone())
    };

    let launch_url = request_launch_from_backend(&server_base_url, &device_id, &normalized_payload)?;
    let launch_url_string = launch_url.to_string();
    write_runtime_log(
        &app,
        &format!("launch_exam_from_client launch_url={launch_url_string}"),
    );

    open_exam_window(&app, &state, launch_url)?;

    Ok(StartExamResponse {
        ok: true,
        server_base_url,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    builder = builder.plugin(tauri_plugin_updater::Builder::new().build());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let global_shortcut_plugin = tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }

                if let Some(state) = app.try_state::<RuntimeState>() {
                    let secret_shortcut_ids = [
                        Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyX).id(),
                        #[cfg(target_os = "macos")]
                        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyX).id(),
                    ];

                    if secret_shortcut_ids
                        .into_iter()
                        .any(|shortcut_id| shortcut.id() == shortcut_id)
                    {
                        if exam_kiosk_is_active(app, &state) {
                            let _ = close_app_with_session_end_runtime(app, &state);
                        }
                        return;
                    }

                    if [
                        Code::F1,
                        Code::F2,
                        Code::F3,
                        Code::F4,
                        Code::F5,
                        Code::F6,
                        Code::F7,
                        Code::F8,
                        Code::F9,
                        Code::F10,
                        Code::F11,
                        Code::F12,
                    ]
                    .into_iter()
                    .any(|code| shortcut.id() == Shortcut::new(None, code).id())
                    {
                        if let Ok(launcher) = launcher_window(app) {
                            ensure_exam_window_kiosk(&launcher);
                            if exam_kiosk_is_active(app, &state) {
                                apply_macos_exam_presentation_lock(app, true);
                            }
                            let _ = launcher.set_focus();
                        }
                        if let Some(exam_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
                            let _ = exam_webview.set_focus();
                        }
                        return;
                    }

                    let refresh_shortcut_ids = [
                        Shortcut::new(Some(Modifiers::CONTROL), Code::KeyR).id(),
                        #[cfg(target_os = "macos")]
                        Shortcut::new(Some(Modifiers::SUPER), Code::KeyR).id(),
                    ];

                    if refresh_shortcut_ids
                        .into_iter()
                        .any(|shortcut_id| shortcut.id() == shortcut_id)
                    {
                        if let Ok(launcher) = launcher_window(app) {
                            ensure_exam_window_kiosk(&launcher);
                            let _ = launcher.set_focus();
                        }
                        if let Some(exam_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
                            let _ = exam_webview.set_focus();
                        }
                        return;
                    }

                    if shortcut.id() == Shortcut::new(None, Code::Escape).id() {
                        if exam_kiosk_is_active(app, &state) {
                            if let Ok(launcher) = launcher_window(app) {
                                ensure_exam_window_kiosk(&launcher);
                                apply_macos_exam_presentation_lock(app, true);
                                let _ = launcher.set_focus();
                            }
                            if let Some(exam_webview) = app.get_webview(EXAM_WEBVIEW_LABEL) {
                                let _ = exam_webview.set_focus();
                            }
                        }
                        return;
                    }

                    if exam_kiosk_is_active(app, &state) {
                        let _ = return_to_launcher_runtime(
                            app,
                            &state,
                            "global shortcut emergency return invoked",
                        );
                    }
                }
            })
            .build();

        builder = builder.plugin(global_shortcut_plugin);
    }

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

            if should_migrate_to_default_server_url(&loaded.server_base_url) {
                loaded.server_base_url = DEFAULT_SERVER_BASE_URL.to_string();
            }

            if loaded.device_id.trim().is_empty() {
                loaded.device_id = generate_device_id();
            }

            persist_client_state(&app.handle(), &loaded)?;

            app.manage(RuntimeState {
                client_state: Mutex::new(loaded),
                allow_exam_close: Mutex::new(true),
                overlay_state: Mutex::new(None),
                heartbeat_stop_signal: Mutex::new(None),
                heartbeat_session_key: Mutex::new(None),
                kiosk_watchdog_stop_signal: Mutex::new(None),
            });

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;

                let shortcuts = [
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyX),
                    Shortcut::new(None, Code::Escape),
                    Shortcut::new(None, Code::F1),
                    Shortcut::new(None, Code::F2),
                    Shortcut::new(None, Code::F3),
                    Shortcut::new(None, Code::F4),
                    Shortcut::new(None, Code::F5),
                    Shortcut::new(None, Code::F6),
                    Shortcut::new(None, Code::F7),
                    Shortcut::new(None, Code::F8),
                    Shortcut::new(None, Code::F9),
                    Shortcut::new(None, Code::F10),
                    Shortcut::new(None, Code::F11),
                    Shortcut::new(None, Code::F12),
                    Shortcut::new(Some(Modifiers::CONTROL), Code::KeyR),
                ];

                for shortcut in shortcuts {
                    let _ = app.global_shortcut().register(shortcut);
                }

                #[cfg(target_os = "macos")]
                {
                    let _ = app
                        .global_shortcut()
                        .register(Shortcut::new(Some(Modifiers::SUPER), Code::KeyR));
                    let _ = app.global_shortcut().register(Shortcut::new(
                        Some(Modifiers::SUPER | Modifiers::SHIFT),
                        Code::KeyX,
                    ));
                }
            }

            set_android_exam_lock(&app.handle(), false);

            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != LAUNCHER_WINDOW_LABEL {
                return;
            }

            let should_sync_exam = matches!(event, WindowEvent::Resized(_) | WindowEvent::Moved(_));
            if should_sync_exam {
                if let Some(state) = window.try_state::<RuntimeState>() {
                    if exam_kiosk_is_active(&window.app_handle(), &state) {
                        let _ = resize_exam_webview(&window.app_handle());
                        if let Ok(launcher) = launcher_window(&window.app_handle()) {
                            ensure_exam_window_kiosk(&launcher);
                        }
                        if let Some(exam_webview) = window.app_handle().get_webview(EXAM_WEBVIEW_LABEL) {
                            let _ = exam_webview.set_focus();
                        }
                        apply_macos_exam_presentation_lock(&window.app_handle(), true);
                    }
                }
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
                        if let Ok(launcher) = launcher_window(&window.app_handle()) {
                            ensure_exam_window_kiosk(&launcher);
                            apply_macos_exam_presentation_lock(&window.app_handle(), true);
                            #[cfg(not(any(target_os = "android", target_os = "ios")))]
                            let _ = launcher.set_focus();
                        }
                        if let Some(exam_webview) = window.app_handle().get_webview(EXAM_WEBVIEW_LABEL) {
                            let _ = exam_webview.set_focus();
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
            launch_exam_from_client,
            open_exam_direct,
            finish_exam_session,
            get_exam_overlay_state,
            execute_secret_exit,
            get_updater_channel_info,
            check_for_updates,
            install_available_update,
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

            if let Ok(launcher) = launcher_window(app_handle) {
                api.prevent_exit();
                ensure_exam_window_kiosk(&launcher);
                if let Some(exam_webview) = app_handle.get_webview(EXAM_WEBVIEW_LABEL) {
                    let _ = exam_webview.set_focus();
                }
                apply_macos_exam_presentation_lock(app_handle, true);
            }
        }
    });
}
