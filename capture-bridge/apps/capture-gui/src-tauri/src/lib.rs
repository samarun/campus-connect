use std::{
    env,
    path::{Path, PathBuf},
    process::{Child, Stdio},
    sync::Mutex,
};

use capture_common::{
    Backend, CaptureMetrics, CaptureState, CaptureStatus, CheckLevel, HealthResponse, PlanRequest,
    PlanResponse, ProfileConfig, ProfileOverrides, ProfileSummary, ProfilesResponse, StartRequest,
    StopResponse, ValidateRequest, ValidationReport, now_unix_seconds,
};
use serde::Serialize;
use tauri::WebviewUrl;

struct GuiState {
    session: Mutex<Session>,
}

#[derive(Default)]
struct Session {
    status: CaptureStatus,
    child: Option<Child>,
}

#[derive(Clone, Debug, Serialize)]
struct DisplayInfo {
    id: usize,
    name: String,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    scale_factor: f64,
}

#[tauri::command]
fn list_formats() -> Vec<capture_common::FormatPreset> {
    capture_core::recommended_formats()
}

#[tauri::command]
fn list_devices(backend: String) -> Result<String, String> {
    let backend = backend
        .parse::<Backend>()
        .map_err(|error| error.to_string())?;
    capture_core::list_devices(backend).map_err(|error| error.to_string())
}

#[tauri::command]
fn list_profiles() -> Result<ProfilesResponse, String> {
    let config = capture_core::load_config(&config_path()).map_err(|error| error.to_string())?;
    let profiles = config
        .profiles
        .iter()
        .map(|(name, profile)| ProfileSummary {
            name: name.clone(),
            purpose: profile.purpose.clone(),
            safe_mode: profile.safe_mode.unwrap_or(false),
        })
        .collect();
    Ok(ProfilesResponse { profiles })
}

#[tauri::command]
fn save_profile(name: String, profile: ProfileConfig) -> Result<ProfilesResponse, String> {
    let config = capture_core::save_profile(&config_path(), &name, profile)
        .map_err(|error| error.to_string())?;
    let profiles = config
        .profiles
        .iter()
        .map(|(name, profile)| ProfileSummary {
            name: name.clone(),
            purpose: profile.purpose.clone(),
            safe_mode: profile.safe_mode.unwrap_or(false),
        })
        .collect();
    Ok(ProfilesResponse { profiles })
}

#[tauri::command]
fn validate_profile(
    profile: String,
    overrides: Option<ProfileOverrides>,
) -> Result<ValidationReport, String> {
    let request = ValidateRequest {
        profile,
        overrides: overrides.unwrap_or_default(),
    };
    let config = capture_core::load_config(&config_path()).map_err(|error| error.to_string())?;
    let resolved = capture_core::resolve_profile(&config, &request.profile, request.overrides)
        .map_err(|error| error.to_string())?;
    Ok(ValidationReport {
        profile: request.profile,
        warnings: capture_core::validate_profile(&resolved),
        checks: capture_core::preflight_profile(&resolved),
    })
}

#[tauri::command]
fn plan_profile(
    profile: String,
    overrides: Option<ProfileOverrides>,
) -> Result<PlanResponse, String> {
    build_plan_response(PlanRequest {
        profile,
        overrides: overrides.unwrap_or_default(),
    })
}

#[tauri::command]
fn current_session(state: tauri::State<'_, GuiState>) -> Result<CaptureStatus, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "capture session lock is unavailable".to_string())?;
    refresh_status(&mut session);
    Ok(session.status.clone())
}

#[tauri::command]
fn start_capture(
    state: tauri::State<'_, GuiState>,
    profile: String,
    overrides: Option<ProfileOverrides>,
    dry_run: bool,
) -> Result<CaptureStatus, String> {
    let request = StartRequest {
        profile,
        overrides: overrides.unwrap_or_default(),
        dry_run,
    };
    let plan_response = build_plan_response(PlanRequest {
        profile: request.profile.clone(),
        overrides: request.overrides,
    })?;

    let failed_checks = plan_response
        .checks
        .iter()
        .filter(|check| matches!(check.level, CheckLevel::Fail))
        .map(|check| format!("{}: {}", check.name, check.message))
        .collect::<Vec<_>>();
    if !failed_checks.is_empty() {
        return Err(format!(
            "preflight failed before capture start: {}",
            failed_checks.join("; ")
        ));
    }

    let mut session = state
        .session
        .lock()
        .map_err(|_| "capture session lock is unavailable".to_string())?;
    refresh_status(&mut session);
    if session.child.is_some() {
        return Err("capture session is already running".to_string());
    }

    let mut status = CaptureStatus {
        state: if request.dry_run {
            CaptureState::Ready
        } else {
            CaptureState::Starting
        },
        selected_profile: Some(request.profile),
        output_url: plan_response.plan.args.last().cloned(),
        started_at_unix_seconds: None,
        metrics: CaptureMetrics::default(),
        last_error: None,
        last_plan: Some(plan_response.plan.clone()),
    };

    if request.dry_run {
        session.status = status.clone();
        return Ok(status);
    }

    which::which(&plan_response.plan.executable)
        .map_err(|_| format!("{} is not available on PATH", plan_response.plan.executable))?;
    let child = std::process::Command::new(&plan_response.plan.executable)
        .args(&plan_response.plan.args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start capture: {error}"))?;

    status.state = CaptureState::Running;
    status.started_at_unix_seconds = Some(now_unix_seconds());
    session.status = status.clone();
    session.child = Some(child);

    Ok(status)
}

#[tauri::command]
fn stop_capture(state: tauri::State<'_, GuiState>) -> Result<StopResponse, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "capture session lock is unavailable".to_string())?;
    let mut stopped = false;
    if let Some(child) = session.child.as_mut() {
        child
            .kill()
            .map_err(|error| format!("failed to stop capture: {error}"))?;
        stopped = true;
    }
    session.child = None;
    session.status.state = CaptureState::Stopped;
    session.status.metrics.uptime_seconds = uptime_seconds(&session.status);
    Ok(StopResponse {
        stopped,
        status: session.status.clone(),
    })
}

#[tauri::command]
fn health(state: tauri::State<'_, GuiState>) -> Result<HealthResponse, String> {
    let mut session = state
        .session
        .lock()
        .map_err(|_| "capture session lock is unavailable".to_string())?;
    refresh_status(&mut session);
    Ok(HealthResponse {
        ok: !matches!(session.status.state, CaptureState::Failed),
        service: "Campus Connect Capture Bridge".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        state: session.status.state,
    })
}

#[tauri::command]
fn config_location() -> String {
    config_path().display().to_string()
}

#[tauri::command]
fn list_displays(app: tauri::AppHandle) -> Result<Vec<DisplayInfo>, String> {
    Ok(app
        .available_monitors()
        .map_err(|error| error.to_string())?
        .into_iter()
        .enumerate()
        .map(|(id, monitor)| {
            let position = monitor.position();
            let size = monitor.size();
            DisplayInfo {
                id,
                name: monitor.name().map_or("Display", |name| name).to_string(),
                x: position.x,
                y: position.y,
                width: size.width,
                height: size.height,
                scale_factor: monitor.scale_factor(),
            }
        })
        .collect::<Vec<_>>())
}

#[tauri::command]
async fn open_campus_view(
    app: tauri::AppHandle,
    route: String,
    title: String,
    display_id: Option<usize>,
    fullscreen: bool,
) -> Result<String, String> {
    let url = campus_view_url(&route)?;
    let label = format!(
        "campus-connect-{}-{}",
        route
            .trim_matches('/')
            .replace(['/', '?', '&', '='], "-")
            .chars()
            .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
            .collect::<String>(),
        now_unix_seconds()
    );

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app,
        label.clone(),
        WebviewUrl::External(
            url.parse()
                .map_err(|error| format!("invalid Campus Connect URL {url}: {error}"))?,
        ),
    )
    .title(title)
    .inner_size(1280.0, 720.0);

    if let Some(display_id) = display_id
        && let Some(display) = app
            .available_monitors()
            .map_err(|error| error.to_string())?
            .into_iter()
            .nth(display_id)
    {
        let position = display.position();
        let size = display.size();
        builder = builder
            .position(position.x as f64, position.y as f64)
            .inner_size(size.width as f64, size.height as f64);
    }

    let window = builder
        .fullscreen(fullscreen)
        .build()
        .map_err(|error| error.to_string())?;
    if fullscreen {
        window
            .set_fullscreen(true)
            .map_err(|error| error.to_string())?;
    }
    Ok(label)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(GuiState {
            session: Mutex::new(Session::default()),
        })
        .invoke_handler(tauri::generate_handler![
            list_formats,
            list_devices,
            list_profiles,
            save_profile,
            validate_profile,
            plan_profile,
            current_session,
            start_capture,
            stop_capture,
            health,
            config_location,
            list_displays,
            open_campus_view
        ])
        .run(tauri::generate_context!())
        .expect("error while running Campus Connect Capture Bridge");
}

fn build_plan_response(request: PlanRequest) -> Result<PlanResponse, String> {
    let config = capture_core::load_config(&config_path()).map_err(|error| error.to_string())?;
    let resolved = capture_core::resolve_profile(&config, &request.profile, request.overrides)
        .map_err(|error| error.to_string())?;
    let plan = capture_core::build_plan(&resolved).map_err(|error| error.to_string())?;
    Ok(PlanResponse {
        plan,
        warnings: capture_core::validate_profile(&resolved),
        checks: capture_core::preflight_profile(&resolved),
    })
}

fn refresh_status(session: &mut Session) {
    if let Some(child) = session.child.as_mut() {
        match child.try_wait() {
            Ok(Some(status)) => {
                session.child = None;
                session.status.metrics.uptime_seconds = uptime_seconds(&session.status);
                if status.success() {
                    session.status.state = CaptureState::Stopped;
                    session.status.last_error = None;
                } else {
                    session.status.state = CaptureState::Failed;
                    session.status.last_error = Some(format!("capture exited with {status}"));
                }
            }
            Ok(None) => {
                session.status.state = CaptureState::Running;
                session.status.metrics.uptime_seconds = uptime_seconds(&session.status);
            }
            Err(error) => {
                session.child = None;
                session.status.state = CaptureState::Failed;
                session.status.last_error = Some(error.to_string());
            }
        }
    }
}

fn uptime_seconds(status: &CaptureStatus) -> u64 {
    status
        .started_at_unix_seconds
        .map(|started_at| now_unix_seconds().saturating_sub(started_at))
        .unwrap_or_default()
}

fn config_path() -> PathBuf {
    if let Some(path) = env::var_os("CAPTURE_BRIDGE_CONFIG").map(PathBuf::from) {
        return path;
    }

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        PathBuf::from("config/capture.local.toml"),
        manifest_dir.join("config/capture.local.toml"),
        manifest_dir
            .join("../../..")
            .join("config/capture.local.toml"),
    ];

    let path = candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .unwrap_or_else(|| candidates[2].clone());

    std::fs::canonicalize(&path).unwrap_or(path)
}

fn campus_view_url(route: &str) -> Result<String, String> {
    let base =
        env::var("CAMPUS_CONNECT_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let base = base.trim_end_matches('/');
    let route = if route.starts_with('/') {
        route.to_string()
    } else {
        format!("/{route}")
    };
    Ok(format!("{base}{route}"))
}
