use std::process::Command;

use anyhow::{Context, Result};
use capture_common::Backend;

pub fn list_devices(backend: Backend) -> Result<String> {
    match backend {
        Backend::Avfoundation => run_ffmpeg_discovery(&[
            "-hide_banner",
            "-f",
            "avfoundation",
            "-list_devices",
            "true",
            "-i",
            "",
        ]),
        Backend::Decklink => run_ffmpeg_discovery(&[
            "-hide_banner",
            "-f",
            "decklink",
            "-list_devices",
            "1",
            "-i",
            "dummy",
        ]),
        Backend::V4l2 => Ok(list_v4l2_hint()),
        Backend::Lavfi => Ok("lavfi test source is available through FFmpeg when lavfi is enabled.\nUse video_device = \"testsrc2\".\n".to_string()),
        Backend::GstreamerDecklink => Ok(run_optional_command(
            "gst-device-monitor-1.0",
            &["Video/Source"],
            "GStreamer DeckLink discovery needs gst-device-monitor-1.0 and DeckLink plugins on the rack host.",
        )),
        Backend::GstreamerV4l2 => Ok(run_optional_command(
            "gst-device-monitor-1.0",
            &["Video/Source"],
            "GStreamer v4l2 discovery needs gst-device-monitor-1.0 on the rack host.",
        )),
        Backend::Aja => Ok("AJA discovery requires the AJA SDK/tools on the capture machine.\nUse the vendor control panel until the native AJA adapter is installed.\n".to_string()),
    }
}

pub fn backend_warnings(backend: Backend) -> Vec<String> {
    let mut warnings = Vec::new();
    match backend {
        Backend::Avfoundation => {
            if !ffmpeg_has_device("avfoundation") {
                warnings.push(
                    "FFmpeg does not report the avfoundation input device on this host."
                        .to_string(),
                );
            }
        }
        Backend::Decklink => {
            if !ffmpeg_has_device("decklink") {
                warnings.push("FFmpeg does not report DeckLink support. Install Blackmagic Desktop Video SDK and an FFmpeg build with decklink enabled.".to_string());
            }
        }
        Backend::V4l2 => {
            if !cfg!(target_os = "linux") {
                warnings.push(
                    "v4l2 is a Linux backend; use avfoundation on macOS or decklink on production capture hosts."
                        .to_string(),
                );
            }
            if !ffmpeg_has_device("v4l2") {
                warnings
                    .push("FFmpeg does not report v4l2 input support on this host.".to_string());
            }
        }
        Backend::Lavfi => {
            if !ffmpeg_has_device("lavfi") {
                warnings.push(
                    "FFmpeg does not report lavfi support for synthetic test sources.".to_string(),
                );
            }
        }
        Backend::GstreamerDecklink => {
            if which::which("gst-launch-1.0").is_err() {
                warnings.push("GStreamer runner needs gst-launch-1.0 on PATH.".to_string());
            }
        }
        Backend::GstreamerV4l2 => {
            if !cfg!(target_os = "linux") {
                warnings.push("gstreamer-v4l2 is intended for Linux rack hosts.".to_string());
            }
            if which::which("gst-launch-1.0").is_err() {
                warnings.push("GStreamer runner needs gst-launch-1.0 on PATH.".to_string());
            }
        }
        Backend::Aja => {
            warnings.push("AJA native SDK backend is a production extension point and is not launched by this runner yet.".to_string());
        }
    }
    warnings
}

pub fn ffmpeg_has_encoder(encoder: &str) -> bool {
    let Ok(output) = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
    else {
        return false;
    };
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text.lines().any(|line| line.contains(encoder))
}

fn run_ffmpeg_discovery(args: &[&str]) -> Result<String> {
    which::which("ffmpeg").context("ffmpeg is required for capture device discovery")?;
    let output = Command::new("ffmpeg")
        .args(args)
        .output()
        .context("failed to run ffmpeg")?;
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    let filtered = text
        .lines()
        .filter(|line| {
            !line.contains("[in#0 @")
                && !line.contains("Error opening input")
                && !line.contains("Error opening input file")
                && !line.contains("Error opening input files")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{filtered}\n"))
}

fn ffmpeg_has_device(device: &str) -> bool {
    let Ok(output) = Command::new("ffmpeg")
        .args(["-hide_banner", "-devices"])
        .output()
    else {
        return false;
    };
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text.lines().any(|line| line.contains(device))
}

fn run_optional_command(command: &str, args: &[&str], fallback: &str) -> String {
    let Ok(output) = Command::new(command).args(args).output() else {
        return format!("{fallback}\n");
    };
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    text
}

fn list_v4l2_hint() -> String {
    if cfg!(target_os = "linux") {
        "Linux v4l2 devices usually appear as /dev/video0, /dev/video1, etc.\nRun `v4l2-ctl --list-devices` for friendly names.\n".to_string()
    } else {
        "v4l2 is for Linux capture hosts. On macOS, use `list-devices avfoundation`.\n".to_string()
    }
}
