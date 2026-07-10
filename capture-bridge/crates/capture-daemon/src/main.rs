use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use capture_common::{
    Backend, CaptureMetrics, CaptureState, CaptureStatus, HealthResponse, PlanRequest,
    PlanResponse, ProfileSummary, ProfilesResponse, SaveProfileRequest, StartRequest, StopResponse,
    ValidateRequest, ValidationReport, now_unix_seconds,
};
use clap::Parser;
use serde::Deserialize;
use tokio::{process::Child, sync::Mutex};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{EnvFilter, fmt};

#[derive(Parser, Debug)]
#[command(
    name = "capture-daemon",
    version,
    about = "Headless Campus Connect Capture Bridge daemon for macOS workstations and Linux rack hosts"
)]
struct Cli {
    #[arg(short, long, default_value = "config/capture.local.toml")]
    config: PathBuf,

    #[arg(long, default_value = "127.0.0.1:4768")]
    bind: SocketAddr,

    #[arg(long, env = "CAPTURE_BRIDGE_API_TOKEN")]
    api_token: Option<String>,
}

#[derive(Clone)]
struct AppState {
    config_path: PathBuf,
    api_token: Option<String>,
    session: Arc<Mutex<Session>>,
}

#[derive(Default)]
struct Session {
    status: CaptureStatus,
    child: Option<Child>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: "unauthorized".to_string(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.message
            })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
struct DevicesQuery {
    backend: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let cli = Cli::parse();
    if !is_loopback(cli.bind.ip()) && cli.api_token.as_deref().unwrap_or("").is_empty() {
        bail!("LAN daemon bind requires CAPTURE_BRIDGE_API_TOKEN or --api-token");
    }

    let state = AppState {
        config_path: cli.config,
        api_token: cli.api_token,
        session: Arc::new(Mutex::new(Session::default())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/devices", get(devices))
        .route("/formats", get(formats))
        .route("/profiles", get(profiles).post(save_profile))
        .route("/validate", post(validate))
        .route("/plan", post(plan))
        .route("/sessions/start", post(start_session))
        .route("/sessions/stop", post(stop_session))
        .route("/sessions/current", get(current_session))
        .route("/metrics", get(metrics))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(cli.bind)
        .await
        .with_context(|| format!("unable to bind daemon to {}", cli.bind))?;
    tracing::info!("Campus Connect Capture Bridge daemon listening on {}", cli.bind);
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let session = state.session.lock().await;
    Json(HealthResponse {
        ok: !matches!(session.status.state, CaptureState::Failed),
        service: "Campus Connect Capture Bridge".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        state: session.status.state,
    })
}

async fn devices(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<DevicesQuery>,
) -> Result<String, ApiError> {
    require_auth(&state, &headers)?;
    let backend = query
        .backend
        .as_deref()
        .unwrap_or("avfoundation")
        .parse::<Backend>()
        .map_err(|error| ApiError::bad_request(error.to_string()))?;
    capture_core::list_devices(backend).map_err(to_bad_request)
}

async fn formats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<capture_common::FormatPreset>>, ApiError> {
    require_auth(&state, &headers)?;
    Ok(Json(capture_core::recommended_formats()))
}

async fn profiles(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ProfilesResponse>, ApiError> {
    require_auth(&state, &headers)?;
    let config = capture_core::load_config(&state.config_path).map_err(to_bad_request)?;
    let profiles = config
        .profiles
        .iter()
        .map(|(name, profile)| ProfileSummary {
            name: name.clone(),
            purpose: profile.purpose.clone(),
            safe_mode: profile.safe_mode.unwrap_or(false),
        })
        .collect();
    Ok(Json(ProfilesResponse { profiles }))
}

async fn save_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<SaveProfileRequest>,
) -> Result<Json<ProfilesResponse>, ApiError> {
    require_auth(&state, &headers)?;
    let config = capture_core::save_profile(&state.config_path, &request.name, request.profile)
        .map_err(to_bad_request)?;
    let profiles = config
        .profiles
        .iter()
        .map(|(name, profile)| ProfileSummary {
            name: name.clone(),
            purpose: profile.purpose.clone(),
            safe_mode: profile.safe_mode.unwrap_or(false),
        })
        .collect();
    Ok(Json(ProfilesResponse { profiles }))
}

async fn validate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidationReport>, ApiError> {
    require_auth(&state, &headers)?;
    let config = capture_core::load_config(&state.config_path).map_err(to_bad_request)?;
    let resolved = capture_core::resolve_profile(&config, &request.profile, request.overrides)
        .map_err(to_bad_request)?;
    Ok(Json(ValidationReport {
        profile: request.profile,
        warnings: capture_core::validate_profile(&resolved),
        checks: capture_core::preflight_profile(&resolved),
    }))
}

async fn plan(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<PlanRequest>,
) -> Result<Json<PlanResponse>, ApiError> {
    require_auth(&state, &headers)?;
    let response = build_plan_response(&state.config_path, request).map_err(to_bad_request)?;
    Ok(Json(response))
}

async fn start_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<StartRequest>,
) -> Result<Json<CaptureStatus>, ApiError> {
    require_auth(&state, &headers)?;
    let plan_response = build_plan_response(
        &state.config_path,
        PlanRequest {
            profile: request.profile.clone(),
            overrides: request.overrides,
        },
    )
    .map_err(to_bad_request)?;

    let mut session = state.session.lock().await;
    refresh_status(&mut session);
    if session.child.is_some() {
        return Err(ApiError::conflict("capture session is already running"));
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
        return Ok(Json(status));
    }

    let child = tokio::process::Command::new(&plan_response.plan.executable)
        .args(&plan_response.plan.args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|error| ApiError::bad_request(format!("failed to start capture: {error}")))?;

    status.state = CaptureState::Running;
    status.started_at_unix_seconds = Some(now_unix_seconds());
    session.status = status.clone();
    session.child = Some(child);

    Ok(Json(status))
}

async fn stop_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StopResponse>, ApiError> {
    require_auth(&state, &headers)?;
    let mut session = state.session.lock().await;
    let mut stopped = false;
    if let Some(child) = session.child.as_mut() {
        child
            .kill()
            .await
            .map_err(|error| ApiError::bad_request(format!("failed to stop capture: {error}")))?;
        stopped = true;
    }
    session.child = None;
    session.status.state = CaptureState::Stopped;
    session.status.metrics.uptime_seconds = uptime_seconds(&session.status);
    Ok(Json(StopResponse {
        stopped,
        status: session.status.clone(),
    }))
}

async fn current_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CaptureStatus>, ApiError> {
    require_auth(&state, &headers)?;
    let mut session = state.session.lock().await;
    refresh_status(&mut session);
    Ok(Json(session.status.clone()))
}

async fn metrics(State(state): State<AppState>) -> String {
    let mut session = state.session.lock().await;
    refresh_status(&mut session);
    let status = &session.status;
    format!(
        "# HELP capture_bridge_running Capture bridge running state\n\
         # TYPE capture_bridge_running gauge\n\
         capture_bridge_running {}\n\
         # HELP capture_bridge_uptime_seconds Capture bridge session uptime\n\
         # TYPE capture_bridge_uptime_seconds gauge\n\
         capture_bridge_uptime_seconds {}\n\
         # HELP capture_bridge_reconnect_count Capture bridge reconnect count\n\
         # TYPE capture_bridge_reconnect_count counter\n\
         capture_bridge_reconnect_count {}\n",
        if matches!(status.state, CaptureState::Running) {
            1
        } else {
            0
        },
        status.metrics.uptime_seconds,
        status.metrics.reconnect_count
    )
}

fn build_plan_response(config_path: &Path, request: PlanRequest) -> Result<PlanResponse> {
    let config = capture_core::load_config(config_path)?;
    let resolved = capture_core::resolve_profile(&config, &request.profile, request.overrides)?;
    let plan = capture_core::build_plan(&resolved)?;
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

fn require_auth(state: &AppState, headers: &HeaderMap) -> Result<(), ApiError> {
    let Some(expected) = state.api_token.as_deref() else {
        return Ok(());
    };
    let bearer = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let header = headers
        .get("x-capture-bridge-token")
        .and_then(|value| value.to_str().ok());
    if bearer == Some(expected) || header == Some(expected) {
        Ok(())
    } else {
        Err(ApiError::unauthorized())
    }
}

fn to_bad_request(error: anyhow::Error) -> ApiError {
    ApiError::bad_request(error.to_string())
}

fn is_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(value) => value.is_loopback(),
        IpAddr::V6(value) => value.is_loopback(),
    }
}
