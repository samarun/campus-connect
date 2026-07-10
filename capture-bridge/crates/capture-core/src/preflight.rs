use std::{
    net::{SocketAddr, ToSocketAddrs},
    path::Path,
};

use capture_common::{CheckLevel, OutputKind, PreflightCheck, ResolvedProfile, VideoEncoder};

use crate::devices::ffmpeg_has_encoder;

pub fn preflight_profile(profile: &ResolvedProfile) -> Vec<PreflightCheck> {
    vec![
        format_check(profile),
        encoder_check(profile),
        output_check(profile),
        recording_path_check(profile),
    ]
}

fn format_check(profile: &ResolvedProfile) -> PreflightCheck {
    if profile.format.width > 1920 || profile.format.height > 1080 {
        return check(
            "format",
            CheckLevel::Fail,
            "Selected format exceeds the Campus Connect 1080p production maximum.",
        );
    }
    check(
        "format",
        CheckLevel::Pass,
        "Selected format is within the Campus Connect 1080p production target.",
    )
}

fn encoder_check(profile: &ResolvedProfile) -> PreflightCheck {
    let encoder = match profile.encoder {
        VideoEncoder::Libx264 => "libx264",
        VideoEncoder::H264Videotoolbox => "h264_videotoolbox",
        VideoEncoder::H264Nvenc => "h264_nvenc",
        VideoEncoder::H264Vaapi => "h264_vaapi",
        VideoEncoder::Copy => {
            return check(
                "encoder",
                CheckLevel::Warn,
                "Video is set to copy; confirm the source is already H.264 compatible with stream relay/RTC.",
            );
        }
    };

    if ffmpeg_has_encoder(encoder) {
        check(
            "encoder",
            CheckLevel::Pass,
            "Selected FFmpeg encoder is available.",
        )
    } else {
        check(
            "encoder",
            CheckLevel::Warn,
            &format!("FFmpeg does not report encoder {encoder} on this host."),
        )
    }
}

fn output_check(profile: &ResolvedProfile) -> PreflightCheck {
    match profile.output.kind {
        OutputKind::Srt | OutputKind::Rtmp | OutputKind::Whip => {
            if let Some((host, port)) = host_port_from_url(&profile.output.url) {
                let addrs = (host.as_str(), port).to_socket_addrs();
                if addrs.is_ok() {
                    check("output", CheckLevel::Pass, "Output host resolves.")
                } else {
                    check(
                        "output",
                        CheckLevel::Warn,
                        "Output host did not resolve from this machine.",
                    )
                }
            } else {
                check(
                    "output",
                    CheckLevel::Warn,
                    "Could not parse output host and port for preflight.",
                )
            }
        }
        OutputKind::File => check("output", CheckLevel::Pass, "File output selected."),
    }
}

fn recording_path_check(profile: &ResolvedProfile) -> PreflightCheck {
    if !matches!(profile.output.kind, OutputKind::File) {
        return check(
            "recording-path",
            CheckLevel::Pass,
            "Recording path check skipped for network output.",
        );
    }

    let path = Path::new(&profile.output.url);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    if parent.exists() {
        check(
            "recording-path",
            CheckLevel::Pass,
            "Recording directory exists.",
        )
    } else {
        check(
            "recording-path",
            CheckLevel::Fail,
            "Recording directory does not exist.",
        )
    }
}

fn host_port_from_url(url: &str) -> Option<(String, u16)> {
    let (_, rest) = url.split_once("://")?;
    let authority = rest.split(['/', '?']).next()?;
    if authority.is_empty() {
        return None;
    }
    let (host, port) = authority.rsplit_once(':')?;
    let parsed_port = port.parse::<u16>().ok()?;
    let socket_candidate = format!("{host}:{parsed_port}");
    if socket_candidate.parse::<SocketAddr>().is_ok() {
        return Some((host.to_string(), parsed_port));
    }
    Some((host.to_string(), parsed_port))
}

fn check(name: &str, level: CheckLevel, message: &str) -> PreflightCheck {
    PreflightCheck {
        name: name.to_string(),
        level,
        message: message.to_string(),
    }
}
