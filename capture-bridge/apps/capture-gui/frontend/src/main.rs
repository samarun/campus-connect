use capture_common::{
    Backend, CaptureState, CaptureStatus, CheckLevel, FormatPreset, PlanResponse, PreflightCheck,
    ProfileConfig, ProfileOverrides, ProfileSummary, ProfilesResponse, StopResponse,
    ValidationReport, VideoEncoder,
};
use leptos::{ev, prelude::*, task::spawn_local};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::json;
use wasm_bindgen::JsCast;

const BACKENDS: [&str; 8] = [
    "",
    "avfoundation",
    "decklink",
    "v4l2",
    "lavfi",
    "gstreamer-decklink",
    "gstreamer-v4l2",
    "aja",
];

const ENCODERS: [&str; 5] = [
    "",
    "libx264",
    "h264-videotoolbox",
    "h264-nvenc",
    "h264-vaapi",
];

const OUTPUTS: [&str; 5] = [
    "",
    "local-program",
    "local-test",
    "production-program",
    "local-recording",
];

#[derive(Clone, Debug, Deserialize)]
struct DisplayInfo {
    id: usize,
    name: String,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug, Deserialize)]
struct PreviewDevice {
    id: String,
    label: String,
}

#[derive(Clone, Debug)]
struct DeviceOption {
    value: String,
    label: String,
}

#[derive(Clone, Copy)]
struct CampusView {
    title: &'static str,
    route: &'static str,
    group: &'static str,
    fullscreen: bool,
}

#[derive(Clone, Copy)]
struct EventRow {
    title: &'static str,
    start: &'static str,
    source: &'static str,
    destinations: &'static str,
    state: &'static str,
}

#[derive(Clone, Copy)]
struct EncoderRow {
    name: &'static str,
    kind: &'static str,
    status: &'static str,
    input: &'static str,
    format: &'static str,
    codec: &'static str,
    bitrate: &'static str,
}

#[derive(Clone, Copy)]
struct StreamRow {
    name: &'static str,
    status: &'static str,
    protocol: &'static str,
    destination: &'static str,
    content: &'static str,
    path: &'static str,
}

#[derive(Clone, Copy)]
struct DecoderRow {
    campus: &'static str,
    status: &'static str,
    mode: &'static str,
    buffer: &'static str,
    output: &'static str,
    cue: &'static str,
}

#[derive(Clone, Copy)]
struct SourceCapability {
    name: &'static str,
    backend: &'static str,
    status: &'static str,
    note: &'static str,
}

#[derive(Clone, Copy)]
struct ServiceRow {
    name: &'static str,
    state: &'static str,
    interface: &'static str,
    purpose: &'static str,
}

const CAMPUS_VIEWS: [CampusView; 11] = [
    CampusView {
        title: "Login",
        route: "/login",
        group: "Access",
        fullscreen: false,
    },
    CampusView {
        title: "Operator Console",
        route: "/operator",
        group: "Access",
        fullscreen: false,
    },
    CampusView {
        title: "Primary Site",
        route: "/campus/main",
        group: "Campus",
        fullscreen: false,
    },
    CampusView {
        title: "Remote Site A",
        route: "/campus/b",
        group: "Campus",
        fullscreen: false,
    },
    CampusView {
        title: "Remote Site B",
        route: "/campus/c",
        group: "Campus",
        fullscreen: false,
    },
    CampusView {
        title: "Primary Return Feed",
        route: "/receiver/main",
        group: "Receiver Outputs",
        fullscreen: true,
    },
    CampusView {
        title: "Remote Site A Full Screen",
        route: "/receiver/campus-b",
        group: "Receiver Outputs",
        fullscreen: true,
    },
    CampusView {
        title: "Remote Site B Full Screen",
        route: "/receiver/campus-c",
        group: "Receiver Outputs",
        fullscreen: true,
    },
    CampusView {
        title: "3-Way Grid",
        route: "/receiver/main?view=grid",
        group: "Receiver Outputs",
        fullscreen: true,
    },
    CampusView {
        title: "Program Player",
        route: "/player/program",
        group: "Receiver Outputs",
        fullscreen: true,
    },
    CampusView {
        title: "Test Tools",
        route: "/test",
        group: "Tools",
        fullscreen: false,
    },
];

const EVENTS: [EventRow; 4] = [
    EventRow {
        title: "Weekend Event - Greeting Mode",
        start: "08:45",
        source: "All sites",
        destinations: "Primary Site, Remote Site A, Remote Site B",
        state: "ready",
    },
    EventRow {
        title: "Weekend Event - Program Relay",
        start: "10:05",
        source: "Primary program",
        destinations: "Remote Site A, Remote Site B, Program Player",
        state: "scheduled",
    },
    EventRow {
        title: "Midweek Program Test",
        start: "Wed 18:30",
        source: "Test pattern",
        destinations: "stream relay broadcast/test",
        state: "draft",
    },
    EventRow {
        title: "Local Rehearsal",
        start: "Manual",
        source: "OBS Virtual Camera",
        destinations: "local stream relay + receiver windows",
        state: "manual",
    },
];

const VIDEO_ENCODERS: [EncoderRow; 3] = [
    EncoderRow {
        name: "Video Encoder 1",
        kind: "Program",
        status: "ready",
        input: "Selected capture source",
        format: "1080p30",
        codec: "H.264",
        bitrate: "9 Mbps",
    },
    EncoderRow {
        name: "Video Encoder 2",
        kind: "Backup",
        status: "standby",
        input: "Test / backup source",
        format: "720p30",
        codec: "H.264",
        bitrate: "4.5 Mbps",
    },
    EncoderRow {
        name: "Video Encoder 3",
        kind: "Future",
        status: "planned",
        input: "RTC WHIP",
        format: "1080p30",
        codec: "H.264 / HEVC later",
        bitrate: "Adaptive",
    },
];

const AUDIO_ENCODERS: [EncoderRow; 4] = [
    EncoderRow {
        name: "Audio 1",
        kind: "Program Mix",
        status: "ready",
        input: "Embedded / selected audio",
        format: "48 kHz stereo",
        codec: "AAC",
        bitrate: "192 kbps",
    },
    EncoderRow {
        name: "Audio 2",
        kind: "Speaker ISO",
        status: "planned",
        input: "Channels 3-4",
        format: "48 kHz stereo",
        codec: "AAC",
        bitrate: "128 kbps",
    },
    EncoderRow {
        name: "Audio 3",
        kind: "Audience",
        status: "planned",
        input: "Channels 5-6",
        format: "48 kHz stereo",
        codec: "AAC",
        bitrate: "128 kbps",
    },
    EncoderRow {
        name: "Audio 4",
        kind: "Translation",
        status: "planned",
        input: "Channels 7-8",
        format: "48 kHz stereo",
        codec: "AAC",
        bitrate: "128 kbps",
    },
];

const STREAMS: [StreamRow; 4] = [
    StreamRow {
        name: "Program Relay",
        status: "ready",
        protocol: "SRT",
        destination: "localhost:9999",
        content: "Video Encoder 1 + Audio 1",
        path: "default/broadcast/program",
    },
    StreamRow {
        name: "stream relay Backup",
        status: "standby",
        protocol: "SRT",
        destination: "localhost:9999",
        content: "Video Encoder 2 + Audio 1",
        path: "default/broadcast/backup",
    },
    StreamRow {
        name: "Web Preview",
        status: "ready",
        protocol: "LL-HLS/WebRTC",
        destination: "stream.example.com",
        content: "broadcast/program playback",
        path: "/player/program",
    },
    StreamRow {
        name: "Greeting Room",
        status: "external",
        protocol: "RTC WebRTC",
        destination: "rtc.example.com",
        content: "campus-greeting",
        path: "browser app",
    },
];

const DECODERS: [DecoderRow; 3] = [
    DecoderRow {
        campus: "Main Confidence",
        status: "optional",
        mode: "Local monitor",
        buffer: "0-5 sec",
        output: "HDMI/SDI receiver window",
        cue: "Operator",
    },
    DecoderRow {
        campus: "Remote Site A",
        status: "ready",
        mode: "LAN first / cloud fallback",
        buffer: "30-60 sec",
        output: "Fullscreen display",
        cue: "Countdown + manual start",
    },
    DecoderRow {
        campus: "Remote Site B",
        status: "ready",
        mode: "LAN first / cloud fallback",
        buffer: "30-60 sec",
        output: "Fullscreen display",
        cue: "Countdown + manual start",
    },
];

const SOURCE_CAPABILITIES: [SourceCapability; 6] = [
    SourceCapability {
        name: "OBS Virtual Camera",
        backend: "AVFoundation / v4l2",
        status: "supported",
        note: "Run OBS virtual camera, then choose it from the scanned device list.",
    },
    SourceCapability {
        name: "Blackmagic DeckLink",
        backend: "FFmpeg decklink / GStreamer",
        status: "supported",
        note: "Best production path for SDI/HDMI capture cards.",
    },
    SourceCapability {
        name: "AJA capture",
        backend: "AJA SDK adapter",
        status: "adapter",
        note: "Vendor SDK integration remains the production extension point.",
    },
    SourceCapability {
        name: "USB/UVC cameras",
        backend: "AVFoundation / v4l2",
        status: "supported",
        note: "Useful for local tests and emergency fallback.",
    },
    SourceCapability {
        name: "NDI Virtual Input",
        backend: "Virtual camera",
        status: "supported",
        note: "Appears as a camera device when NDI tools are installed.",
    },
    SourceCapability {
        name: "Synthetic test pattern",
        backend: "lavfi",
        status: "supported",
        note: "Use for stream relay/SRT rehearsal without hardware.",
    },
];

const SERVICES: [ServiceRow; 7] = [
    ServiceRow {
        name: "SRT ingest",
        state: "enabled",
        interface: "9999/udp",
        purpose: "stream relay program and backup relay",
    },
    ServiceRow {
        name: "RTMP ingest",
        state: "enabled",
        interface: "1936/tcp",
        purpose: "Legacy encoder fallback",
    },
    ServiceRow {
        name: "RTC",
        state: "external",
        interface: "rtc.example.com",
        purpose: "Greeting Mode WebRTC",
    },
    ServiceRow {
        name: "Campus web",
        state: "local",
        interface: "localhost:3000",
        purpose: "Operator and receiver views",
    },
    ServiceRow {
        name: "Daemon API",
        state: "local-only",
        interface: "127.0.0.1",
        purpose: "Rack control API",
    },
    ServiceRow {
        name: "SSH",
        state: "admin-only",
        interface: "rack host",
        purpose: "Maintenance access",
    },
    ServiceRow {
        name: "Audit logs",
        state: "planned",
        interface: "TLS syslog",
        purpose: "Security and operations trail",
    },
];

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let (page, set_page) = signal("Dashboard".to_string());
    let (profiles, set_profiles) = signal(Vec::<ProfileSummary>::new());
    let (formats, set_formats) = signal(Vec::<FormatPreset>::new());
    let (selected_profile, set_selected_profile) = signal(String::new());
    let (profile_save_name, set_profile_save_name) = signal("custom-program".to_string());
    let (selected_format, set_selected_format) = signal("1080p30".to_string());
    let (selected_backend, set_selected_backend) = signal(String::new());
    let (selected_encoder, set_selected_encoder) = signal(String::new());
    let (selected_output, set_selected_output) = signal(String::new());
    let (video_device, set_video_device) = signal(String::new());
    let (audio_device, set_audio_device) = signal(String::new());
    let (bitrate, set_bitrate) = signal("9000".to_string());
    let (safe_controls, set_safe_controls) = signal(true);
    let (status, set_status) = signal(CaptureStatus::default());
    let (command, set_command) = signal("Loading Campus Connect Capture Bridge plan...".to_string());
    let (warnings, set_warnings) = signal(Vec::<String>::new());
    let (checks, set_checks) = signal(Vec::<PreflightCheck>::new());
    let (devices, set_devices) = signal(String::new());
    let (video_options, set_video_options) = signal(Vec::<DeviceOption>::new());
    let (audio_options, set_audio_options) = signal(Vec::<DeviceOption>::new());
    let (display_options, set_display_options) = signal(Vec::<DisplayInfo>::new());
    let (selected_display, set_selected_display) = signal(0usize);
    let (preview_devices, set_preview_devices) = signal(Vec::<PreviewDevice>::new());
    let (selected_preview_device, set_selected_preview_device) = signal(String::new());
    let (preview_message, set_preview_message) = signal("Preview stopped".to_string());
    let (logs, set_logs) = signal(Vec::<String>::new());
    let (message, set_message) = signal("Ready".to_string());
    let (config_path, set_config_path) = signal(String::new());
    let (dark_mode, set_dark_mode) = signal(false);
    let (busy, set_busy) = signal(false);

    Effect::new(move |_| {
        spawn_local(async move {
            match invoke_command::<Vec<FormatPreset>>("list_formats", json!({})).await {
                Ok(items) => set_formats.set(items),
                Err(error) => push_log(set_logs, format!("Format load failed: {error}")),
            }

            match invoke_command::<ProfilesResponse>("list_profiles", json!({})).await {
                Ok(response) => {
                    let default_profile = response
                        .profiles
                        .iter()
                        .find(|profile| profile.name == "program-local-srt")
                        .or_else(|| {
                            response
                                .profiles
                                .iter()
                                .find(|profile| profile.name == "program-test-pattern")
                        })
                        .or_else(|| response.profiles.first())
                        .map(|profile| profile.name.clone())
                        .unwrap_or_default();
                    set_profiles.set(response.profiles);
                    set_selected_profile.set(default_profile);
                }
                Err(error) => {
                    set_message.set("Unable to load profiles".to_string());
                    push_log(set_logs, format!("Profile load failed: {error}"));
                }
            }

            match invoke_command::<String>("config_location", json!({})).await {
                Ok(path) => set_config_path.set(path),
                Err(error) => push_log(set_logs, format!("Config path unavailable: {error}")),
            }

            match invoke_command::<Vec<DisplayInfo>>("list_displays", json!({})).await {
                Ok(displays) => set_display_options.set(displays),
                Err(error) => push_log(set_logs, format!("Display discovery failed: {error}")),
            }

            match enumerate_preview_devices().await {
                Ok(devices) => {
                    if let Some(first) = devices.first() {
                        set_selected_preview_device.set(first.id.clone());
                    }
                    set_preview_devices.set(devices);
                }
                Err(error) => push_log(
                    set_logs,
                    format!("Preview device discovery failed: {error}"),
                ),
            }

            set_devices.set("Scanning default AVFoundation devices...".to_string());
            scan_devices_from_ui(
                "avfoundation".to_string(),
                set_devices,
                set_video_device,
                set_audio_device,
                set_video_options,
                set_audio_options,
                set_logs,
            );

            refresh_status(set_status, set_message, set_logs).await;
        });
    });

    Effect::new(move |_| {
        let profile = selected_profile.get();
        if profile.is_empty() {
            return;
        }
        let overrides = collect_overrides(
            selected_format.get(),
            selected_backend.get(),
            selected_encoder.get(),
            selected_output.get(),
            video_device.get(),
            audio_device.get(),
            bitrate.get(),
        );
        spawn_local(async move {
            match invoke_command::<PlanResponse>(
                "plan_profile",
                json!({ "profile": profile, "overrides": overrides }),
            )
            .await
            {
                Ok(response) => {
                    set_command.set(response.plan.shell_line());
                    set_warnings.set(response.warnings);
                    set_checks.set(response.checks);
                    set_message.set("Plan ready".to_string());
                }
                Err(error) => {
                    set_command.set(format!("Plan unavailable: {error}"));
                    set_warnings.set(vec![error]);
                    set_checks.set(Vec::new());
                    set_message.set("Plan error".to_string());
                }
            }
        });
    });

    let run_validation = move |_| {
        let profile = selected_profile.get();
        let overrides = collect_overrides(
            selected_format.get(),
            selected_backend.get(),
            selected_encoder.get(),
            selected_output.get(),
            video_device.get(),
            audio_device.get(),
            bitrate.get(),
        );
        set_busy.set(true);
        spawn_local(async move {
            match invoke_command::<ValidationReport>(
                "validate_profile",
                json!({ "profile": profile, "overrides": overrides }),
            )
            .await
            {
                Ok(report) => {
                    set_warnings.set(report.warnings);
                    set_checks.set(report.checks);
                    set_message.set("Validation complete".to_string());
                    push_log(set_logs, "Validation complete");
                }
                Err(error) => {
                    set_message.set("Validation failed".to_string());
                    push_log(set_logs, format!("Validation failed: {error}"));
                }
            }
            set_busy.set(false);
        });
    };

    let run_dry_start = move |_| {
        start_capture_from_ui(
            true,
            selected_profile.get(),
            collect_overrides(
                selected_format.get(),
                selected_backend.get(),
                selected_encoder.get(),
                selected_output.get(),
                video_device.get(),
                audio_device.get(),
                bitrate.get(),
            ),
            set_status,
            set_message,
            set_logs,
            set_busy,
        );
    };

    let run_start = move |_| {
        start_capture_from_ui(
            false,
            selected_profile.get(),
            collect_overrides(
                selected_format.get(),
                selected_backend.get(),
                selected_encoder.get(),
                selected_output.get(),
                video_device.get(),
                audio_device.get(),
                bitrate.get(),
            ),
            set_status,
            set_message,
            set_logs,
            set_busy,
        );
    };

    let run_stop = move |_| {
        set_busy.set(true);
        spawn_local(async move {
            match invoke_command::<StopResponse>("stop_capture", json!({})).await {
                Ok(response) => {
                    set_status.set(response.status);
                    set_message.set(if response.stopped {
                        "Capture stopped".to_string()
                    } else {
                        "No capture process was running".to_string()
                    });
                    push_log(set_logs, "Stop requested");
                }
                Err(error) => {
                    set_message.set("Stop failed".to_string());
                    push_log(set_logs, format!("Stop failed: {error}"));
                }
            }
            set_busy.set(false);
        });
    };

    let run_device_scan = move |_| {
        let backend = selected_backend.get();
        let backend = if backend.is_empty() {
            "avfoundation".to_string()
        } else {
            backend
        };
        set_devices.set("Scanning devices...".to_string());
        scan_devices_from_ui(
            backend,
            set_devices,
            set_video_device,
            set_audio_device,
            set_video_options,
            set_audio_options,
            set_logs,
        );
    };

    let run_profile_save = move |_| {
        let name = profile_save_name.get();
        let profile = ProfileConfig {
            purpose: Some("program".to_string()),
            backend: non_empty(selected_backend.get())
                .and_then(|value| value.parse::<Backend>().ok()),
            video_device: non_empty(video_device.get()),
            audio_device: non_empty(audio_device.get()),
            format: non_empty(selected_format.get()),
            output: non_empty(selected_output.get()),
            encoder: non_empty(selected_encoder.get())
                .and_then(|value| value.parse::<VideoEncoder>().ok()),
            safe_mode: Some(safe_controls.get()),
            macos: None,
            linux: None,
        };
        set_busy.set(true);
        spawn_local(async move {
            match invoke_command::<ProfilesResponse>(
                "save_profile",
                json!({ "name": name.clone(), "profile": profile }),
            )
            .await
            {
                Ok(response) => {
                    set_profiles.set(response.profiles);
                    set_selected_profile.set(name.clone());
                    set_message.set("Profile saved".to_string());
                    push_log(set_logs, format!("Saved profile {name}"));
                }
                Err(error) => {
                    set_message.set("Profile save failed".to_string());
                    push_log(set_logs, format!("Profile save failed: {error}"));
                }
            }
            set_busy.set(false);
        });
    };

    let run_preview_start = move |_| {
        let device_id = selected_preview_device.get();
        set_preview_message.set("Starting preview...".to_string());
        spawn_local(async move {
            match start_preview("capture-preview-video", &device_id).await {
                Ok(()) => {
                    set_preview_message.set("Preview running".to_string());
                    push_log(set_logs, "Preview started");
                }
                Err(error) => {
                    set_preview_message.set("Preview failed".to_string());
                    push_log(set_logs, format!("Preview failed: {error}"));
                }
            }
        });
    };

    let run_preview_stop = move |_| {
        stop_preview("capture-preview-video");
        set_preview_message.set("Preview stopped".to_string());
        push_log(set_logs, "Preview stopped");
    };

    let run_preview_refresh = move |_| {
        set_preview_message.set("Refreshing cameras...".to_string());
        spawn_local(async move {
            match enumerate_preview_devices().await {
                Ok(devices) => {
                    if let Some(first) = devices.first() {
                        set_selected_preview_device.set(first.id.clone());
                    }
                    let count = devices.len();
                    set_preview_devices.set(devices);
                    set_preview_message.set(if count == 0 {
                        "No preview cameras found".to_string()
                    } else {
                        format!("{count} preview camera(s) found")
                    });
                    push_log(set_logs, format!("Preview camera refresh found {count} device(s)"));
                }
                Err(error) => {
                    set_preview_message.set("Camera refresh failed".to_string());
                    push_log(set_logs, format!("Preview camera refresh failed: {error}"));
                }
            }
        });
    };

    view! {
        <div class=move || if dark_mode.get() { "app theme-dark" } else { "app theme-light" }>
            <header class="topbar">
                <div class="brand">
                    <div class="mark">"CC"</div>
                    <div>
                        <p class="eyebrow">"CAMPUS CONNECT"</p>
                        <h1>"Campus Connect Capture Bridge"</h1>
                    </div>
                </div>
                <div class="topbar-actions">
                    <button
                        class="theme-toggle"
                        on:click=move |_| set_dark_mode.update(|value| *value = !*value)
                    >
                        {move || if dark_mode.get() { "Light Mode" } else { "Dark Mode" }}
                    </button>
                    <span class=move || format!("status {}", state_class(status.get().state))>
                        <span class="dot"></span>{move || status_label(status.get().state)}
                    </span>
                </div>
            </header>
            <div class="layout">
                <nav class="nav">
                    <p class="nav-label">"Appliance"</p>
                    {["Dashboard", "Monitor"].into_iter().map(|item| {
                        let item_text = item.to_string();
                        view! {
                            <button class=move || if page.get() == item_text { "active" } else { "" } on:click=move |_| set_page.set(item.to_string())>{item}</button>
                        }
                    }).collect_view()}
                    <p class="nav-label">"Campus"</p>
                    {["Connect", "Events", "Decoders"].into_iter().map(|item| {
                        let item_text = item.to_string();
                        view! {
                            <button class=move || if page.get() == item_text { "active" } else { "" } on:click=move |_| set_page.set(item.to_string())>{item}</button>
                        }
                    }).collect_view()}
                    <p class="nav-label">"Capture"</p>
                    {["Source", "Format", "Encoders"].into_iter().map(|item| {
                        let item_text = item.to_string();
                        view! {
                            <button class=move || if page.get() == item_text { "active" } else { "" } on:click=move |_| set_page.set(item.to_string())>{item}</button>
                        }
                    }).collect_view()}
                    <p class="nav-label">"Streaming"</p>
                    {["Streams", "Output"].into_iter().map(|item| {
                        let item_text = item.to_string();
                        view! {
                            <button class=move || if page.get() == item_text { "active" } else { "" } on:click=move |_| set_page.set(item.to_string())>{item}</button>
                        }
                    }).collect_view()}
                    <p class="nav-label">"Admin"</p>
                    {["Network", "Security", "Profiles", "Logs", "System"].into_iter().map(|item| {
                        let item_text = item.to_string();
                        view! {
                            <button class=move || if page.get() == item_text { "active" } else { "" } on:click=move |_| set_page.set(item.to_string())>{item}</button>
                        }
                    }).collect_view()}
                </nav>
                <section class="content">
                    <div class="page-title">
                        <div>
                            <p class="eyebrow">"CAPTURE STATION"</p>
                            <h2>{move || format!("{} · Campus Connect Capture Bridge", page.get())}</h2>
                        </div>
                        <p class="message">{move || message.get()}</p>
                    </div>

                    <Show when=move || page.get() == "Connect">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Campus Connect Desktop Views"</h3>
                                <p class="muted">"Open the existing Campus Connect web app as desktop windows. Receiver outputs can be sent fullscreen to any connected display."</p>
                                <label>"Target Display For Fullscreen Outputs"
                                    <select on:change=move |event| {
                                        let value = event_target_value(&event).parse::<usize>().unwrap_or(0);
                                        set_selected_display.set(value);
                                    }>
                                        {move || {
                                            let displays = display_options.get();
                                            if displays.is_empty() {
                                                view! { <option value="0">"Primary Display"</option> }.into_any()
                                            } else {
                                                displays.into_iter().map(|display| {
                                                    let value = display.id.to_string();
                                                    view! {
                                                        <option value=value.clone() selected=move || selected_display.get().to_string() == value>
                                                            {format!("{} · {}x{}", display.name, display.width, display.height)}
                                                        </option>
                                                    }
                                                }).collect_view().into_any()
                                            }
                                        }}
                                    </select>
                                </label>
                            </section>

                            <section class="panel">
                                <h3>"Access"</h3>
                                <div class="view-list">
                                    {CAMPUS_VIEWS.into_iter().filter(|view| view.group == "Access").map(|view| {
                                        view! {
                                            <div class="view-row">
                                                <strong>{view.title}</strong>
                                                <button class="button secondary" on:click=move |_| open_campus_view_from_ui(view, false, selected_display, set_logs)>"Open"</button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Remote Site Bonsoles"</h3>
                                <div class="view-list">
                                    {CAMPUS_VIEWS.into_iter().filter(|view| view.group == "Campus").map(|view| {
                                        view! {
                                            <div class="view-row">
                                                <strong>{view.title}</strong>
                                                <button class="button secondary" on:click=move |_| open_campus_view_from_ui(view, false, selected_display, set_logs)>"Open"</button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel wide">
                                <h3>"Receiver Outputs"</h3>
                                <div class="receiver-grid">
                                    {CAMPUS_VIEWS.into_iter().filter(|view| view.group == "Receiver Outputs").map(|view| {
                                        view! {
                                            <div class="receiver-card">
                                                <strong>{view.title}</strong>
                                                <span>{view.route}</span>
                                                <div class="actions">
                                                    <button class="button secondary" on:click=move |_| open_campus_view_from_ui(view, false, selected_display, set_logs)>"Window"</button>
                                                    <button class="button" on:click=move |_| open_campus_view_from_ui(view, true, selected_display, set_logs)>"Fullscreen"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Tools"</h3>
                                <div class="view-list">
                                    {CAMPUS_VIEWS.into_iter().filter(|view| view.group == "Tools").map(|view| {
                                        view! {
                                            <div class="view-row">
                                                <strong>{view.title}</strong>
                                                <button class="button secondary" on:click=move |_| open_campus_view_from_ui(view, false, selected_display, set_logs)>"Open"</button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Events">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Service Events"</h3>
                                <p class="muted">"Schedule-like control for greeting, program relay, rehearsals, and test patterns. This mirrors the workflow volunteers expect from dedicated multisite systems while staying local-first."</p>
                                <div class="event-list">
                                    {EVENTS.into_iter().map(|event| {
                                        view! {
                                            <div class="event-card">
                                                <div>
                                                    <strong>{event.title}</strong>
                                                    <span>{format!("{} · {} -> {}", event.start, event.source, event.destinations)}</span>
                                                </div>
                                                <span class=format!("status-pill {}", event.state)>{event.state}</span>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Next Run"</h3>
                                <dl class="metrics">
                                    <div><dt>"Mode"</dt><dd>"Greeting then Program"</dd></div>
                                    <div><dt>"Default Source"</dt><dd>{move || selected_profile.get()}</dd></div>
                                    <div><dt>"Cue"</dt><dd>"Manual / countdown"</dd></div>
                                    <div><dt>"Fallback"</dt><dd>"720p30 test pattern"</dd></div>
                                </dl>
                                <div class="actions">
                                    <button class="button secondary" disabled=true>"Add Event"</button>
                                    <button class="button secondary" disabled=true>"Duplicate"</button>
                                </div>
                            </section>

                            <section class="panel wide">
                                <h3>"Volunteer Run Sheet"</h3>
                                <div class="run-sheet">
                                    <div><strong>"1"</strong><span>"Open campus consoles and confirm Main, Remote Site A, and Remote Site B are online."</span></div>
                                    <div><strong>"2"</strong><span>"Start local preview and confirm the correct OBS/capture-card source."</span></div>
                                    <div><strong>"3"</strong><span>"Dry Run the selected profile, then start capture when preflight is clean."</span></div>
                                    <div><strong>"4"</strong><span>"Send receiver windows fullscreen to connected displays before handoff."</span></div>
                                </div>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Dashboard">
                        <div class="grid">
                            <section class="panel">
                                <h3>"Session Control"</h3>
                                <dl class="metrics">
                                    <div><dt>"State"</dt><dd>{move || status_label(status.get().state)}</dd></div>
                                    <div><dt>"Profile"</dt><dd>{move || status.get().selected_profile.unwrap_or_else(|| selected_profile.get())}</dd></div>
                                    <div><dt>"Uptime"</dt><dd>{move || format!("{}s", status.get().metrics.uptime_seconds)}</dd></div>
                                    <div><dt>"Dropped Frames"</dt><dd>{move || status.get().metrics.dropped_frames.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string())}</dd></div>
                                </dl>
                                <div class="actions">
                                    <button class="button" disabled=move || busy.get() on:click=run_start>"Start"</button>
                                    <button class="button danger" disabled=move || busy.get() on:click=run_stop>"Stop"</button>
                                    <button class="button secondary" disabled=move || busy.get() on:click=run_dry_start>"Dry Run"</button>
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Production Profile"</h3>
                                <label>"Profile"
                                    <select on:change=move |event| set_selected_profile.set(event_target_value(&event))>
                                        {move || profiles.get().into_iter().map(|profile| {
                                            let value = profile.name.clone();
                                            view! {
                                                <option value=value.clone() selected=move || selected_profile.get() == value>
                                                    {format!("{}{}", profile.name, if profile.safe_mode { " · safe" } else { "" })}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        checked=move || safe_controls.get()
                                        on:change=move |event| set_safe_controls.set(event_target_checked(&event))
                                    />
                                    <span>"Volunteer-safe controls"</span>
                                </label>
                            </section>

                            <section class="panel">
                                <h3>"Selected Format"</h3>
                                <div class="hero-number">{move || selected_format.get()}</div>
                                <p class="muted">"Default production target is 1080p30. Use Format page for frame-rate changes."</p>
                            </section>

                            <section class="panel wide">
                                <h3>"Appliance Overview"</h3>
                                <div class="appliance-strip">
                                    <div>
                                        <strong>"Input"</strong>
                                        <span>{move || if video_device.get().is_empty() { "Auto-scanning".to_string() } else { video_device.get() }}</span>
                                    </div>
                                    <div>
                                        <strong>"Encoder"</strong>
                                        <span>{move || if selected_encoder.get().is_empty() { "Profile default".to_string() } else { selected_encoder.get() }}</span>
                                    </div>
                                    <div>
                                        <strong>"Stream"</strong>
                                        <span>"SRT to stream relay"</span>
                                    </div>
                                    <div>
                                        <strong>"Decoders"</strong>
                                        <span>"Remote Site A + Remote Site B"</span>
                                    </div>
                                </div>
                            </section>

                            <section class="panel wide">
                                <h3>"Generated Media Command"</h3>
                                <textarea readonly prop:value=move || command.get()></textarea>
                            </section>

                            <section class="panel">
                                <h3>"Preflight"</h3>
                                <button class="button secondary" disabled=move || busy.get() on:click=run_validation>"Validate"</button>
                                <ul class="checks compact-list">
                                    {move || checks.get().into_iter().map(|check| {
                                        view! {
                                            <li class=check_class(check.level)>
                                                <strong>{check.name}</strong>
                                                <span>{check.message}</span>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            </section>

                            <section class="panel">
                                <h3>"Warnings"</h3>
                                <WarningsList warnings=warnings />
                            </section>

                            <section class="panel wide">
                                <h3>"Recent Logs"</h3>
                                <LogsList logs=logs />
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Source">
                        <div class="grid">
                            <section class="panel">
                                <h3>"Capture Backend"</h3>
                                <label>"Backend"
                                    <select on:change=move |event| set_selected_backend.set(event_target_value(&event))>
                                        {BACKENDS.into_iter().map(|backend| {
                                            view! {
                                                <option value=backend selected=move || selected_backend.get() == backend>
                                                    {if backend.is_empty() { "Profile default" } else { backend }}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <p class="muted">"macOS uses AVFoundation/DeckLink. Linux rack uses v4l2 or GStreamer."</p>
                            </section>

                            <section class="panel">
                                <h3>"Video Source"</h3>
                                <Show
                                    when=move || !video_options.get().is_empty()
                                    fallback=move || view! {
                                        <label>"Video Device"
                                            <input
                                                placeholder="1, /dev/video0, DeckLink Mini Recorder 4K"
                                                value=move || video_device.get()
                                                on:input=move |event| set_video_device.set(event_target_value(&event))
                                            />
                                        </label>
                                    }
                                >
                                    <label>"Video Device"
                                        <select on:change=move |event| set_video_device.set(event_target_value(&event))>
                                            {move || video_options.get().into_iter().map(|device| {
                                                let value = device.value.clone();
                                                view! {
                                                    <option value=value.clone() selected=move || video_device.get() == value>
                                                        {device.label}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </label>
                                </Show>
                            </section>

                            <section class="panel">
                                <h3>"Audio Source"</h3>
                                <Show
                                    when=move || !audio_options.get().is_empty()
                                    fallback=move || view! {
                                        <label>"Audio Device"
                                            <input
                                                placeholder="2, default, hw:0"
                                                value=move || audio_device.get()
                                                on:input=move |event| set_audio_device.set(event_target_value(&event))
                                            />
                                        </label>
                                    }
                                >
                                    <label>"Audio Device"
                                        <select on:change=move |event| set_audio_device.set(event_target_value(&event))>
                                            {move || audio_options.get().into_iter().map(|device| {
                                                let value = device.value.clone();
                                                view! {
                                                    <option value=value.clone() selected=move || audio_device.get() == value>
                                                        {device.label}
                                                    </option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </label>
                                </Show>
                            </section>

                            <section class="panel wide">
                                <h3>"Device Discovery"</h3>
                                <button class="button secondary" on:click=run_device_scan>"Scan Devices"</button>
                                <pre class="device-output">{move || if devices.get().is_empty() { "Choose a backend, then scan devices.".to_string() } else { devices.get() }}</pre>
                            </section>

                            <section class="panel wide">
                                <h3>"Local Camera Preview"</h3>
                                <label>"Preview Device"
                                    <select on:change=move |event| set_selected_preview_device.set(event_target_value(&event))>
                                        {move || {
                                            let devices = preview_devices.get();
                                            if devices.is_empty() {
                                                view! { <option value="">"Default camera"</option> }.into_any()
                                            } else {
                                                devices.into_iter().map(|device| {
                                                    let value = device.id.clone();
                                                    view! {
                                                        <option value=value.clone() selected=move || selected_preview_device.get() == value>
                                                            {device.label}
                                                        </option>
                                                    }
                                                }).collect_view().into_any()
                                            }
                                        }}
                                    </select>
                                </label>
                                <div class="preview-shell">
                                    <video id="capture-preview-video" autoplay=true playsinline=true muted=true></video>
                                </div>
                                <div class="actions">
                                    <button class="button secondary" on:click=run_preview_refresh>"Refresh Cameras"</button>
                                    <button class="button" on:click=run_preview_start>"Start Preview"</button>
                                    <button class="button secondary" on:click=run_preview_stop>"Stop Preview"</button>
                                    <span class="inline-status">{move || preview_message.get()}</span>
                                </div>
                            </section>

                            <section class="panel wide">
                                <h3>"Supported Capture Sources"</h3>
                                <div class="source-grid">
                                    {SOURCE_CAPABILITIES.into_iter().map(|source| {
                                        view! {
                                            <div class="source-card">
                                                <div>
                                                    <strong>{source.name}</strong>
                                                    <span>{source.backend}</span>
                                                </div>
                                                <span class=format!("status-pill {}", source.status)>{source.status}</span>
                                                <p>{source.note}</p>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Format">
                        <div class="grid">
                            <section class="panel">
                                <h3>"Format / Frame Rate"</h3>
                                <label>"Production Format"
                                    <select on:change=move |event| set_selected_format.set(event_target_value(&event))>
                                        {move || formats.get().into_iter().map(|format| {
                                            let value = format.name.clone();
                                            view! {
                                                <option value=value.clone() selected=move || selected_format.get() == value>
                                                    {format!("{} · {}x{} @ {}", format.name, format.width, format.height, format.frame_rate)}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <p class="muted">"Higher than 1080p30 is allowed, but the bridge warns before production use."</p>
                            </section>

                            <section class="panel">
                                <h3>"Encoding Rate"</h3>
                                <label>"Video Bitrate (kbps)"
                                    <input
                                        value=move || bitrate.get()
                                        on:input=move |event| set_bitrate.set(event_target_value(&event))
                                    />
                                </label>
                            </section>

                            <section class="panel wide">
                                <h3>"Current Plan"</h3>
                                <textarea readonly prop:value=move || command.get()></textarea>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Encoders">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Video Encoders"</h3>
                                <div class="list-toolbar">
                                    <span class="muted">"Independent encoder rows with start/stop, settings, and statistics controls."</span>
                                    <div class="actions">
                                        <button class="button secondary" disabled=true>"Start Selected"</button>
                                        <button class="button secondary" disabled=true>"Stop Selected"</button>
                                    </div>
                                </div>
                                <div class="appliance-list">
                                    {VIDEO_ENCODERS.into_iter().map(|encoder| {
                                        view! {
                                            <div class="appliance-row">
                                                <input type="checkbox" />
                                                <span class=format!("status-pill {}", encoder.status)>{encoder.status}</span>
                                                <div>
                                                    <strong>{encoder.name}</strong>
                                                    <span>{encoder.kind}</span>
                                                </div>
                                                <div><small>"Input"</small><span>{encoder.input}</span></div>
                                                <div><small>"Format"</small><span>{encoder.format}</span></div>
                                                <div><small>"Codec"</small><span>{encoder.codec}</span></div>
                                                <div><small>"Bitrate"</small><span>{encoder.bitrate}</span></div>
                                                <div class="actions compact-actions">
                                                    <button class="button secondary" disabled=true>"Stats"</button>
                                                    <button class="button secondary" disabled=true>"Settings"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel wide">
                                <h3>"Audio Encoders"</h3>
                                <div class="appliance-list">
                                    {AUDIO_ENCODERS.into_iter().map(|encoder| {
                                        view! {
                                            <div class="appliance-row">
                                                <input type="checkbox" />
                                                <span class=format!("status-pill {}", encoder.status)>{encoder.status}</span>
                                                <div>
                                                    <strong>{encoder.name}</strong>
                                                    <span>{encoder.kind}</span>
                                                </div>
                                                <div><small>"Input"</small><span>{encoder.input}</span></div>
                                                <div><small>"Mode"</small><span>{encoder.format}</span></div>
                                                <div><small>"Codec"</small><span>{encoder.codec}</span></div>
                                                <div><small>"Bitrate"</small><span>{encoder.bitrate}</span></div>
                                                <div class="actions compact-actions">
                                                    <button class="button secondary" disabled=true>"Mute"</button>
                                                    <button class="button secondary" disabled=true>"Stats"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Active Preset"</h3>
                                <dl class="metrics">
                                    <div><dt>"Profile"</dt><dd>{move || selected_profile.get()}</dd></div>
                                    <div><dt>"Format"</dt><dd>{move || selected_format.get()}</dd></div>
                                    <div><dt>"Bitrate"</dt><dd>{move || format!("{} kbps", bitrate.get())}</dd></div>
                                    <div><dt>"Safe Mode"</dt><dd>{move || if safe_controls.get() { "On" } else { "Off" }}</dd></div>
                                </dl>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Streams">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Streams"</h3>
                                <p class="muted">"Stream rows follow the appliance pattern: status, protocol, destination, selected content, route, and direct controls."</p>
                                <div class="appliance-list">
                                    {STREAMS.into_iter().map(|stream| {
                                        view! {
                                            <div class="appliance-row stream-row">
                                                <input type="checkbox" />
                                                <span class=format!("status-pill {}", stream.status)>{stream.status}</span>
                                                <div>
                                                    <strong>{stream.name}</strong>
                                                    <span>{stream.protocol}</span>
                                                </div>
                                                <div><small>"Destination"</small><span>{stream.destination}</span></div>
                                                <div><small>"Content"</small><span>{stream.content}</span></div>
                                                <div><small>"Path"</small><span>{stream.path}</span></div>
                                                <div class="actions compact-actions">
                                                    <button class="button secondary" disabled=true>"Stats"</button>
                                                    <button class="button secondary" disabled=true>"Settings"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"SRT Relay"</h3>
                                <dl class="metrics">
                                    <div><dt>"Port"</dt><dd>"9999/udp"</dd></div>
                                    <div><dt>"Latency"</dt><dd>"120 ms default"</dd></div>
                                    <div><dt>"Streams"</dt><dd>"broadcast/program, broadcast/backup, broadcast/test"</dd></div>
                                    <div><dt>"Mode"</dt><dd>"caller -> stream relay"</dd></div>
                                </dl>
                            </section>

                            <section class="panel wide">
                                <h3>"Generated Stream Command"</h3>
                                <textarea readonly prop:value=move || command.get()></textarea>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Decoders">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Campus Decoders / Receiver Outputs"</h3>
                                <p class="muted">"These controls map to receiver windows today and can map to dedicated rack decoders later."</p>
                                <div class="decoder-grid">
                                    {DECODERS.into_iter().map(|decoder| {
                                        view! {
                                            <div class="decoder-card">
                                                <div class="decoder-head">
                                                    <strong>{decoder.campus}</strong>
                                                    <span class=format!("status-pill {}", decoder.status)>{decoder.status}</span>
                                                </div>
                                                <dl class="metrics">
                                                    <div><dt>"Mode"</dt><dd>{decoder.mode}</dd></div>
                                                    <div><dt>"Buffer"</dt><dd>{decoder.buffer}</dd></div>
                                                    <div><dt>"Output"</dt><dd>{decoder.output}</dd></div>
                                                    <div><dt>"Cue"</dt><dd>{decoder.cue}</dd></div>
                                                </dl>
                                                <div class="actions">
                                                    <button class="button secondary" disabled=true>"Pause"</button>
                                                    <button class="button secondary" disabled=true>"Cue"</button>
                                                    <button class="button secondary" disabled=true>"Countdown"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Fullscreen Outputs"</h3>
                                <div class="view-list">
                                    {CAMPUS_VIEWS.into_iter().filter(|view| view.group == "Receiver Outputs").map(|view| {
                                        view! {
                                            <div class="view-row">
                                                <strong>{view.title}</strong>
                                                <button class="button" on:click=move |_| open_campus_view_from_ui(view, true, selected_display, set_logs)>"Fullscreen"</button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Output">
                        <div class="grid">
                            <section class="panel">
                                <h3>"Encoder"</h3>
                                <label>"Video Encoder"
                                    <select on:change=move |event| set_selected_encoder.set(event_target_value(&event))>
                                        {ENCODERS.into_iter().map(|encoder| {
                                            view! {
                                                <option value=encoder selected=move || selected_encoder.get() == encoder>
                                                    {if encoder.is_empty() { "Profile default" } else { encoder }}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </label>
                            </section>

                            <section class="panel">
                                <h3>"Target"</h3>
                                <label>"Output Target"
                                    <select on:change=move |event| set_selected_output.set(event_target_value(&event))>
                                        {OUTPUTS.into_iter().map(|output| {
                                            view! {
                                                <option value=output selected=move || selected_output.get() == output>
                                                    {if output.is_empty() { "Profile default" } else { output }}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </label>
                            </section>

                            <section class="panel">
                                <h3>"Program Path"</h3>
                                <dl class="metrics">
                                    <div><dt>"Mode"</dt><dd>"Program SRT"</dd></div>
                                    <div><dt>"Default"</dt><dd>"stream relay"</dd></div>
                                    <div><dt>"Stream"</dt><dd>"broadcast/program"</dd></div>
                                </dl>
                            </section>

                            <section class="panel wide">
                                <h3>"Generated Output Command"</h3>
                                <textarea readonly prop:value=move || command.get()></textarea>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Monitor">
                        <div class="grid">
                            <section class="panel">
                                <h3>"Runtime Status"</h3>
                                <dl class="metrics">
                                    <div><dt>"State"</dt><dd>{move || status_label(status.get().state)}</dd></div>
                                    <div><dt>"Output"</dt><dd>{move || status.get().output_url.unwrap_or_else(|| "-".to_string())}</dd></div>
                                    <div><dt>"FPS"</dt><dd>{move || status.get().metrics.fps.map(|value| format!("{value:.2}")).unwrap_or_else(|| "-".to_string())}</dd></div>
                                    <div><dt>"Bitrate"</dt><dd>{move || status.get().metrics.bitrate_kbps.map(|value| format!("{value} kbps")).unwrap_or_else(|| "-".to_string())}</dd></div>
                                    <div><dt>"Reconnects"</dt><dd>{move || status.get().metrics.reconnect_count.to_string()}</dd></div>
                                </dl>
                            </section>

                            <section class="panel">
                                <h3>"Preflight Checks"</h3>
                                <ul class="checks">
                                    {move || checks.get().into_iter().map(|check| {
                                        view! {
                                            <li class=check_class(check.level)>
                                                <strong>{check.name}</strong>
                                                <span>{check.message}</span>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            </section>

                            <section class="panel">
                                <h3>"Warnings"</h3>
                                <WarningsList warnings=warnings />
                            </section>

                            <section class="panel wide">
                                <h3>"Last Generated Plan"</h3>
                                <textarea readonly prop:value=move || command.get()></textarea>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Network">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Network Services"</h3>
                                <p class="muted">"Keep only required services enabled on rack hosts. LAN-first program delivery and local-only control are the production defaults."</p>
                                <div class="appliance-list">
                                    {SERVICES.into_iter().map(|service| {
                                        view! {
                                            <div class="appliance-row service-row">
                                                <span class=format!("status-pill {}", service.state)>{service.state}</span>
                                                <div>
                                                    <strong>{service.name}</strong>
                                                    <span>{service.purpose}</span>
                                                </div>
                                                <div><small>"Interface"</small><span>{service.interface}</span></div>
                                                <div class="actions compact-actions">
                                                    <button class="button secondary" disabled=true>"Toggle"</button>
                                                    <button class="button secondary" disabled=true>"Details"</button>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </section>

                            <section class="panel">
                                <h3>"Interfaces"</h3>
                                <dl class="metrics">
                                    <div><dt>"Control"</dt><dd>"127.0.0.1 default"</dd></div>
                                    <div><dt>"LAN ingest"</dt><dd>"eth0 / primary NIC"</dd></div>
                                    <div><dt>"Cloud relay"</dt><dd>"stream.example.com"</dd></div>
                                    <div><dt>"TURN"</dt><dd>"turn.example.com"</dd></div>
                                </dl>
                            </section>

                            <section class="panel">
                                <h3>"Preflight Targets"</h3>
                                <ul class="checks compact-list">
                                    <li class="check pass"><strong>"DNS"</strong><span>"Resolve app, RTC, stream relay, and TURN names before service."</span></li>
                                    <li class="check warn"><strong>"Latency"</strong><span>"Keep SRT low enough for service flow, high enough for stability."</span></li>
                                    <li class="check pass"><strong>"Firewall"</strong><span>"Allow SRT 9999/udp and RTC/TURN ports only where required."</span></li>
                                </ul>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Security">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Accounts and Roles"</h3>
                                <ul class="profile-list">
                                    <li>
                                        <strong>"admin"</strong>
                                        <span>"Full configuration"</span>
                                        <em>"enabled"</em>
                                    </li>
                                    <li>
                                        <strong>"operator"</strong>
                                        <span>"Start/stop, outputs, events"</span>
                                        <em>"enabled"</em>
                                    </li>
                                    <li>
                                        <strong>"main"</strong>
                                        <span>"Main campus console"</span>
                                        <em>"enabled"</em>
                                    </li>
                                    <li>
                                        <strong>"campus-b / campus-c"</strong>
                                        <span>"Campus receiver and greeting"</span>
                                        <em>"enabled"</em>
                                    </li>
                                </ul>
                            </section>

                            <section class="panel">
                                <h3>"Control API"</h3>
                                <dl class="metrics">
                                    <div><dt>"Default Bind"</dt><dd>"Unix socket / 127.0.0.1"</dd></div>
                                    <div><dt>"LAN Control"</dt><dd>"Token required"</dd></div>
                                    <div><dt>"Secrets"</dt><dd>"Not stored in profiles"</dd></div>
                                    <div><dt>"Session"</dt><dd>"Idle timeout planned"</dd></div>
                                </dl>
                            </section>

                            <section class="panel">
                                <h3>"Audit Trail"</h3>
                                <ul class="logs">
                                    <li>"Record start, stop, dry-run, profile save, and output launch actions."</li>
                                    <li>"Send rack daemon logs to local rotating files first; TLS syslog later."</li>
                                    <li>"Expose Prometheus metrics for Grafana production dashboards."</li>
                                </ul>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Profiles">
                        <div class="grid">
                            <section class="panel">
                                <h3>"Load Profile"</h3>
                                <label>"Profile"
                                    <select on:change=move |event| set_selected_profile.set(event_target_value(&event))>
                                        {move || profiles.get().into_iter().map(|profile| {
                                            let value = profile.name.clone();
                                            view! {
                                                <option value=value.clone() selected=move || selected_profile.get() == value>
                                                    {format!("{}{}", profile.name, if profile.safe_mode { " · safe" } else { "" })}
                                                </option>
                                            }
                                        }).collect_view()}
                                    </select>
                                </label>
                            </section>

                            <section class="panel">
                                <h3>"Save Current Settings"</h3>
                                <label>"Save As"
                                    <input
                                        value=move || profile_save_name.get()
                                        on:input=move |event| set_profile_save_name.set(event_target_value(&event))
                                    />
                                </label>
                                <button class="button secondary" disabled=move || busy.get() on:click=run_profile_save>
                                    "Save Profile"
                                </button>
                            </section>

                            <section class="panel wide">
                                <h3>"Profile List"</h3>
                                <ul class="profile-list">
                                    {move || profiles.get().into_iter().map(|profile| {
                                        view! {
                                            <li>
                                                <strong>{profile.name}</strong>
                                                <span>{profile.purpose.unwrap_or_else(|| "capture".to_string())}</span>
                                                <em>{if profile.safe_mode { "safe" } else { "advanced" }}</em>
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "Logs">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"Session Logs"</h3>
                                <LogsList logs=logs />
                            </section>
                            <section class="panel">
                                <h3>"Current Message"</h3>
                                <p>{move || message.get()}</p>
                            </section>
                        </div>
                    </Show>

                    <Show when=move || page.get() == "System">
                        <div class="grid">
                            <section class="panel wide">
                                <h3>"System Paths"</h3>
                                <dl class="metrics">
                                    <div><dt>"Config"</dt><dd>{move || config_path.get()}</dd></div>
                                    <div><dt>"Frontend"</dt><dd>"Rust/WASM Leptos"</dd></div>
                                    <div><dt>"Desktop"</dt><dd>"Tauri 2"</dd></div>
                                </dl>
                            </section>
                            <section class="panel">
                                <h3>"Rack Mode"</h3>
                                <p class="muted">"Linux rack hosts should run the daemon through systemd and connect locally or through the token-protected API."</p>
                            </section>
                            <section class="panel">
                                <h3>"Production Defaults"</h3>
                                <dl class="metrics">
                                    <div><dt>"Format"</dt><dd>"1080p30"</dd></div>
                                    <div><dt>"Audio"</dt><dd>"AAC 48 kHz"</dd></div>
                                    <div><dt>"Relay"</dt><dd>"SRT to stream relay"</dd></div>
                                </dl>
                            </section>
                        </div>
                    </Show>
                </section>
            </div>
        </div>
    }
}

#[component]
fn WarningsList(warnings: ReadSignal<Vec<String>>) -> impl IntoView {
    view! {
        <ul class="warnings">
            {move || {
                let warnings = warnings.get();
                if warnings.is_empty() {
                    view! { <li class="muted">"No active warnings."</li> }.into_any()
                } else {
                    warnings.into_iter().map(|warning| view! { <li>{warning}</li> }).collect_view().into_any()
                }
            }}
        </ul>
    }
}

#[component]
fn LogsList(logs: ReadSignal<Vec<String>>) -> impl IntoView {
    view! {
        <ul class="logs">
            {move || {
                let lines = logs.get();
                if lines.is_empty() {
                    view! { <li class="muted">"No log entries yet."</li> }.into_any()
                } else {
                    lines.into_iter().map(|line| view! { <li>{line}</li> }).collect_view().into_any()
                }
            }}
        </ul>
    }
}

fn collect_overrides(
    format: String,
    backend: String,
    encoder: String,
    output: String,
    video_device: String,
    audio_device: String,
    bitrate: String,
) -> ProfileOverrides {
    ProfileOverrides {
        format: non_empty(format),
        backend: non_empty(backend).and_then(|value| value.parse::<Backend>().ok()),
        encoder: non_empty(encoder).and_then(|value| value.parse::<VideoEncoder>().ok()),
        output: non_empty(output),
        video_device: non_empty(video_device),
        audio_device: non_empty(audio_device),
        video_bitrate_kbps: bitrate.parse::<u32>().ok(),
        frame_rate: None,
    }
}

fn open_campus_view_from_ui(
    view: CampusView,
    force_fullscreen: bool,
    selected_display: ReadSignal<usize>,
    set_logs: WriteSignal<Vec<String>>,
) {
    let fullscreen = force_fullscreen || view.fullscreen;
    let display_id = if fullscreen {
        Some(selected_display.get())
    } else {
        None
    };
    spawn_local(async move {
        match invoke_command::<String>(
            "open_campus_view",
            json!({
                "route": view.route,
                "title": format!("Campus Connect {}", view.title),
                "displayId": display_id,
                "fullscreen": fullscreen
            }),
        )
        .await
        {
            Ok(_) => push_log(set_logs, format!("Opened {}", view.title)),
            Err(error) => push_log(set_logs, format!("Open {} failed: {error}", view.title)),
        }
    });
}

fn scan_devices_from_ui(
    backend: String,
    set_devices: WriteSignal<String>,
    set_video_device: WriteSignal<String>,
    set_audio_device: WriteSignal<String>,
    set_video_options: WriteSignal<Vec<DeviceOption>>,
    set_audio_options: WriteSignal<Vec<DeviceOption>>,
    set_logs: WriteSignal<Vec<String>>,
) {
    spawn_local(async move {
        match invoke_command::<String>("list_devices", json!({ "backend": backend })).await {
            Ok(output) => {
                let (video, audio) = parse_device_options(&output);
                if let Some(first) = video.first() {
                    set_video_device.set(first.value.clone());
                }
                if let Some(first) = audio.first() {
                    set_audio_device.set(first.value.clone());
                }
                let video_count = video.len();
                let audio_count = audio.len();
                set_video_options.set(video);
                set_audio_options.set(audio);
                set_devices.set(output);
                push_log(
                    set_logs,
                    format!("Device scan complete: {video_count} video, {audio_count} audio"),
                );
            }
            Err(error) => {
                set_devices.set(format!("Device scan failed: {error}"));
                push_log(set_logs, format!("Device scan failed: {error}"));
            }
        }
    });
}

fn parse_device_options(output: &str) -> (Vec<DeviceOption>, Vec<DeviceOption>) {
    let mut video = Vec::new();
    let mut audio = Vec::new();
    let mut section = "";

    for line in output.lines() {
        if line.contains("video devices:") {
            section = "video";
            continue;
        }
        if line.contains("audio devices:") {
            section = "audio";
            continue;
        }

        if let Some((value, label)) = parse_indexed_device_line(line) {
            let option = DeviceOption { value, label };
            match section {
                "video" => video.push(option),
                "audio" => audio.push(option),
                _ => {}
            }
        } else if line.contains("/dev/video") {
            let value = line.split_whitespace().next().unwrap_or(line).to_string();
            video.push(DeviceOption {
                label: line.trim().to_string(),
                value,
            });
        }
    }

    (video, audio)
}

fn parse_indexed_device_line(line: &str) -> Option<(String, String)> {
    let mut search_from = 0;
    let mut device = None;

    while let Some(open_offset) = line[search_from..].find('[') {
        let start = search_from + open_offset;
        let Some(close_offset) = line[start + 1..].find(']') else {
            break;
        };
        let end = start + 1 + close_offset;
        let value = line[start + 1..end].trim();
        if !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit()) {
            device = Some((value.to_string(), end));
        }
        search_from = end + 1;
    }

    let (value, end) = device?;
    let label = line[end + 1..].trim();
    if label.is_empty() {
        None
    } else {
        Some((value.clone(), format!("[{value}] {label}")))
    }
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn start_capture_from_ui(
    dry_run: bool,
    profile: String,
    overrides: ProfileOverrides,
    set_status: WriteSignal<CaptureStatus>,
    set_message: WriteSignal<String>,
    set_logs: WriteSignal<Vec<String>>,
    set_busy: WriteSignal<bool>,
) {
    set_busy.set(true);
    spawn_local(async move {
        let command = if dry_run { "dry run" } else { "start" };
        match invoke_command::<CaptureStatus>(
            "start_capture",
            json!({ "profile": profile, "overrides": overrides, "dryRun": dry_run }),
        )
        .await
        {
            Ok(next_status) => {
                set_status.set(next_status);
                set_message.set(if dry_run {
                    "Dry run ready".to_string()
                } else {
                    "Capture running".to_string()
                });
                push_log(set_logs, format!("Capture {command} accepted"));
            }
            Err(error) => {
                set_message.set(format!("Capture {command} failed"));
                push_log(set_logs, format!("Capture {command} failed: {error}"));
            }
        }
        set_busy.set(false);
    });
}

async fn refresh_status(
    set_status: WriteSignal<CaptureStatus>,
    set_message: WriteSignal<String>,
    set_logs: WriteSignal<Vec<String>>,
) {
    match invoke_command::<CaptureStatus>("current_session", json!({})).await {
        Ok(next_status) => {
            set_status.set(next_status);
            set_message.set("Session status loaded".to_string());
        }
        Err(error) => {
            push_log(set_logs, format!("Status load failed: {error}"));
        }
    }
}

fn push_log(set_logs: WriteSignal<Vec<String>>, line: impl Into<String>) {
    set_logs.update(|logs| {
        logs.insert(0, line.into());
        logs.truncate(80);
    });
}

fn status_label(state: CaptureState) -> &'static str {
    match state {
        CaptureState::Idle => "Idle",
        CaptureState::Validating => "Validating",
        CaptureState::Ready => "Ready",
        CaptureState::Starting => "Starting",
        CaptureState::Running => "Running",
        CaptureState::Degraded => "Degraded",
        CaptureState::Failed => "Failed",
        CaptureState::Stopped => "Stopped",
    }
}

fn state_class(state: CaptureState) -> &'static str {
    match state {
        CaptureState::Running => "running",
        CaptureState::Ready => "ready",
        CaptureState::Failed => "failed",
        CaptureState::Degraded => "degraded",
        CaptureState::Starting | CaptureState::Validating => "working",
        CaptureState::Idle | CaptureState::Stopped => "idle",
    }
}

fn check_class(level: CheckLevel) -> &'static str {
    match level {
        CheckLevel::Pass => "check pass",
        CheckLevel::Warn => "check warn",
        CheckLevel::Fail => "check fail",
    }
}

fn event_target_value(event: &ev::Event) -> String {
    leptos::ev::Event::target(event)
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .or_else(|| {
            leptos::ev::Event::target(event)
                .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|select| select.value())
        })
        .unwrap_or_default()
}

fn event_target_checked(event: &ev::Event) -> bool {
    leptos::ev::Event::target(event)
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.checked())
        .unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
async fn enumerate_preview_devices() -> Result<Vec<PreviewDevice>, String> {
    let value = js_enumerate_preview_devices().await;
    let json = value
        .as_string()
        .ok_or_else(|| "preview device enumeration returned no data".to_string())?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn enumerate_preview_devices() -> Result<Vec<PreviewDevice>, String> {
    Ok(Vec::new())
}

#[cfg(target_arch = "wasm32")]
async fn start_preview(element_id: &str, device_id: &str) -> Result<(), String> {
    let value = js_start_preview(element_id, device_id).await;
    if value.as_bool().unwrap_or(false) {
        Ok(())
    } else {
        Err(value
            .as_string()
            .unwrap_or_else(|| "unable to start preview".to_string()))
    }
}

#[cfg(not(target_arch = "wasm32"))]
async fn start_preview(_element_id: &str, _device_id: &str) -> Result<(), String> {
    Err("preview is only available in the desktop webview".to_string())
}

#[cfg(target_arch = "wasm32")]
fn stop_preview(element_id: &str) {
    js_stop_preview(element_id);
}

#[cfg(not(target_arch = "wasm32"))]
fn stop_preview(_element_id: &str) {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
export async function js_enumerate_preview_devices() {
  if (!navigator.mediaDevices?.enumerateDevices) {
    return "[]";
  }
  try {
    const probe = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
    probe.getTracks().forEach((track) => track.stop());
  } catch (_) {
    // Permission may be denied until the user starts preview. Still return what labels are available.
  }
  const devices = await navigator.mediaDevices.enumerateDevices();
  return JSON.stringify(devices
    .filter((device) => device.kind === "videoinput")
    .map((device, index) => ({
      id: device.deviceId,
      label: device.label || `Camera ${index + 1}`
    })));
}

export async function js_start_preview(elementId, deviceId) {
  try {
    const video = document.getElementById(elementId);
    if (!video) return "Preview video element is missing";
    if (video.srcObject) {
      video.srcObject.getTracks().forEach((track) => track.stop());
    }
    const constraints = {
      audio: false,
      video: deviceId ? { deviceId: { exact: deviceId } } : true
    };
    const stream = await navigator.mediaDevices.getUserMedia(constraints);
    video.srcObject = stream;
    await video.play();
    return true;
  } catch (error) {
    return error?.message || String(error);
  }
}

export function js_stop_preview(elementId) {
  const video = document.getElementById(elementId);
  if (video?.srcObject) {
    video.srcObject.getTracks().forEach((track) => track.stop());
    video.srcObject = null;
  }
}
"#)]
extern "C" {
    async fn js_enumerate_preview_devices() -> wasm_bindgen::JsValue;
    async fn js_start_preview(element_id: &str, device_id: &str) -> wasm_bindgen::JsValue;
    fn js_stop_preview(element_id: &str);
}

#[cfg(target_arch = "wasm32")]
async fn invoke_command<T>(command: &str, args: serde_json::Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
        async fn tauri_invoke(command: &str, args: JsValue) -> JsValue;
    }

    let args = serde_wasm_bindgen::to_value(&args).map_err(|error| error.to_string())?;
    let value = tauri_invoke(command, args).await;
    serde_wasm_bindgen::from_value(value).map_err(|error| error.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn invoke_command<T>(command: &str, _args: serde_json::Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    Err(format!(
        "Tauri command '{command}' is only available in the desktop runtime"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_avfoundation_prefixed_device_lines() {
        let output = r#"
[AVFoundation indev @ 0x842c54000] AVFoundation video devices:
[AVFoundation indev @ 0x842c54000] [0] FaceTime HD Camera
[AVFoundation indev @ 0x842c54000] [1] OBS Virtual Camera
[AVFoundation indev @ 0x842c54000] AVFoundation audio devices:
[AVFoundation indev @ 0x842c54000] [0] Loopback Audio
[AVFoundation indev @ 0x842c54000] [2] MacBook Pro Microphone
"#;

        let (video, audio) = parse_device_options(output);

        assert_eq!(video[0].value, "0");
        assert_eq!(video[0].label, "[0] FaceTime HD Camera");
        assert_eq!(video[1].value, "1");
        assert_eq!(audio[0].value, "0");
        assert_eq!(audio[1].label, "[2] MacBook Pro Microphone");
    }
}
