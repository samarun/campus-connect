use std::{collections::BTreeMap, fmt, str::FromStr, time::SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Error)]
#[error("unsupported value '{value}' for {kind}")]
pub struct ParseEnumError {
    kind: &'static str,
    value: String,
}

impl ParseEnumError {
    fn new(kind: &'static str, value: &str) -> Self {
        Self {
            kind,
            value: value.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Backend {
    #[default]
    Avfoundation,
    Decklink,
    V4l2,
    Lavfi,
    GstreamerDecklink,
    GstreamerV4l2,
    Aja,
}

impl Backend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Avfoundation => "avfoundation",
            Self::Decklink => "decklink",
            Self::V4l2 => "v4l2",
            Self::Lavfi => "lavfi",
            Self::GstreamerDecklink => "gstreamer-decklink",
            Self::GstreamerV4l2 => "gstreamer-v4l2",
            Self::Aja => "aja",
        }
    }
}

impl fmt::Display for Backend {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Backend {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "avfoundation" => Ok(Self::Avfoundation),
            "decklink" => Ok(Self::Decklink),
            "v4l2" => Ok(Self::V4l2),
            "lavfi" => Ok(Self::Lavfi),
            "gstreamer-decklink" => Ok(Self::GstreamerDecklink),
            "gstreamer-v4l2" => Ok(Self::GstreamerV4l2),
            "aja" => Ok(Self::Aja),
            other => Err(ParseEnumError::new("backend", other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VideoEncoder {
    #[default]
    Libx264,
    H264Videotoolbox,
    H264Nvenc,
    H264Vaapi,
    Copy,
}

impl VideoEncoder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Libx264 => "libx264",
            Self::H264Videotoolbox => "h264-videotoolbox",
            Self::H264Nvenc => "h264-nvenc",
            Self::H264Vaapi => "h264-vaapi",
            Self::Copy => "copy",
        }
    }
}

impl fmt::Display for VideoEncoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for VideoEncoder {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "libx264" => Ok(Self::Libx264),
            "h264-videotoolbox" => Ok(Self::H264Videotoolbox),
            "h264-nvenc" => Ok(Self::H264Nvenc),
            "h264-vaapi" => Ok(Self::H264Vaapi),
            "copy" => Ok(Self::Copy),
            other => Err(ParseEnumError::new("video encoder", other)),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputKind {
    Srt,
    Rtmp,
    File,
    Whip,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScanMode {
    #[default]
    Progressive,
    InterlacedTff,
    InterlacedBff,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CaptureConfig {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub formats: BTreeMap<String, FormatConfig>,
    #[serde(default)]
    pub outputs: BTreeMap<String, OutputConfig>,
    #[serde(default)]
    pub profiles: BTreeMap<String, ProfileConfig>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Defaults {
    pub backend: Option<Backend>,
    pub video_device: Option<String>,
    pub audio_device: Option<String>,
    pub format: Option<String>,
    pub output: Option<String>,
    pub encoder: Option<VideoEncoder>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FormatConfig {
    pub width: u32,
    pub height: u32,
    pub frame_rate: String,
    #[serde(default)]
    pub scan: ScanMode,
    pub pixel_format: Option<String>,
    pub decklink_format_code: Option<String>,
    pub video_bitrate_kbps: Option<u32>,
    pub keyframe_seconds: Option<f32>,
    pub audio_sample_rate: Option<u32>,
    pub audio_channels: Option<u8>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OutputConfig {
    pub kind: OutputKind,
    pub url: String,
    pub srt_latency_ms: Option<u32>,
    pub srt_mode: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PlatformProfileOverrides {
    pub backend: Option<Backend>,
    pub video_device: Option<String>,
    pub audio_device: Option<String>,
    pub format: Option<String>,
    pub output: Option<String>,
    pub encoder: Option<VideoEncoder>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfileConfig {
    pub purpose: Option<String>,
    pub backend: Option<Backend>,
    pub video_device: Option<String>,
    pub audio_device: Option<String>,
    pub format: Option<String>,
    pub output: Option<String>,
    pub encoder: Option<VideoEncoder>,
    pub safe_mode: Option<bool>,
    pub macos: Option<PlatformProfileOverrides>,
    pub linux: Option<PlatformProfileOverrides>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProfileOverrides {
    pub backend: Option<Backend>,
    pub video_device: Option<String>,
    pub audio_device: Option<String>,
    pub format: Option<String>,
    pub frame_rate: Option<String>,
    pub output: Option<String>,
    pub encoder: Option<VideoEncoder>,
    pub video_bitrate_kbps: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolvedProfile {
    pub name: String,
    pub purpose: Option<String>,
    pub backend: Backend,
    pub video_device: String,
    pub audio_device: Option<String>,
    pub format_name: String,
    pub format: FormatConfig,
    pub output_name: String,
    pub output: OutputConfig,
    pub encoder: VideoEncoder,
    pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormatPreset {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: String,
    pub scan: ScanMode,
    pub typical_bitrate_kbps: String,
    pub production_default: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandPlan {
    pub executable: String,
    pub args: Vec<String>,
}

impl CommandPlan {
    pub fn shell_line(&self) -> String {
        std::iter::once(self.executable.as_str())
            .chain(self.args.iter().map(String::as_str))
            .map(shell_quote)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckLevel {
    Pass,
    Warn,
    Fail,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreflightCheck {
    pub name: String,
    pub level: CheckLevel,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationReport {
    pub profile: String,
    pub warnings: Vec<String>,
    pub checks: Vec<PreflightCheck>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CaptureState {
    #[default]
    Idle,
    Validating,
    Ready,
    Starting,
    Running,
    Degraded,
    Failed,
    Stopped,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CaptureMetrics {
    pub uptime_seconds: u64,
    pub bitrate_kbps: Option<u32>,
    pub fps: Option<f64>,
    pub dropped_frames: Option<u64>,
    pub encoder_speed: Option<f64>,
    pub audio_sample_rate: Option<u32>,
    pub reconnect_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub selected_profile: Option<String>,
    pub output_url: Option<String>,
    pub started_at_unix_seconds: Option<u64>,
    pub metrics: CaptureMetrics,
    pub last_error: Option<String>,
    pub last_plan: Option<CommandPlan>,
}

impl Default for CaptureStatus {
    fn default() -> Self {
        Self {
            state: CaptureState::Idle,
            selected_profile: None,
            output_url: None,
            started_at_unix_seconds: None,
            metrics: CaptureMetrics::default(),
            last_error: None,
            last_plan: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub service: String,
    pub version: String,
    pub state: CaptureState,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub name: String,
    pub purpose: Option<String>,
    pub safe_mode: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfilesResponse {
    pub profiles: Vec<ProfileSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidateRequest {
    pub profile: String,
    #[serde(default)]
    pub overrides: ProfileOverrides,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanRequest {
    pub profile: String,
    #[serde(default)]
    pub overrides: ProfileOverrides,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanResponse {
    pub plan: CommandPlan,
    pub warnings: Vec<String>,
    pub checks: Vec<PreflightCheck>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveProfileRequest {
    pub name: String,
    pub profile: ProfileConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartRequest {
    pub profile: String,
    #[serde(default)]
    pub overrides: ProfileOverrides,
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StopResponse {
    pub stopped: bool,
    pub status: CaptureStatus,
}

pub fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn shell_quote(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "-_./:=+,".contains(ch))
    {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
