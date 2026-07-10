use anyhow::{Result, bail};
use capture_common::{Backend, CommandPlan, OutputKind, ResolvedProfile, VideoEncoder};

use crate::devices;

pub fn build_plan(profile: &ResolvedProfile) -> Result<CommandPlan> {
    match profile.backend {
        Backend::Avfoundation | Backend::Decklink | Backend::V4l2 | Backend::Lavfi => {
            build_ffmpeg_plan(profile)
        }
        Backend::GstreamerDecklink | Backend::GstreamerV4l2 => build_gstreamer_plan(profile),
        Backend::Aja => bail!(
            "aja backend requires native AJA SDK integration; this version does not launch AJA SDK pipelines"
        ),
    }
}

pub fn validate_profile(profile: &ResolvedProfile) -> Vec<String> {
    let mut warnings = devices::backend_warnings(profile.backend);

    if profile.format.width > 1920 || profile.format.height > 1080 {
        warnings.push(
            "Campus Connect production target is 1080p maximum. Avoid 4K for this stack.".to_string(),
        );
    }
    if let Some(fps) = frame_rate_as_f64(&profile.format.frame_rate) {
        if fps > 30.0 {
            warnings.push("Frame rates above 30 fps require more encoder CPU, upload bandwidth, and receiver decode headroom.".to_string());
        }
    } else {
        warnings.push(format!(
            "frame rate '{}' is not recognized",
            profile.format.frame_rate
        ));
    }
    if matches!(profile.output.kind, OutputKind::Whip) {
        warnings.push("WHIP output is reserved for the next RTC hardware-ingest phase. Use SRT to stream relay for program program today.".to_string());
    }
    if profile.encoder == VideoEncoder::H264Videotoolbox && !cfg!(target_os = "macos") {
        warnings.push("h264-videotoolbox is macOS-specific. Use libx264, h264-nvenc, or h264-vaapi on Linux rack hosts.".to_string());
    }
    if matches!(
        profile.encoder,
        VideoEncoder::H264Nvenc | VideoEncoder::H264Vaapi
    ) && !cfg!(target_os = "linux")
    {
        warnings.push(
            "h264-nvenc and h264-vaapi are Linux rack encoder choices, not macOS choices."
                .to_string(),
        );
    }

    warnings
}

fn build_ffmpeg_plan(profile: &ResolvedProfile) -> Result<CommandPlan> {
    let mut args = vec![
        "-hide_banner".to_string(),
        "-nostdin".to_string(),
        "-loglevel".to_string(),
        "info".to_string(),
    ];

    add_ffmpeg_input(profile, &mut args)?;
    add_ffmpeg_encoding(profile, &mut args)?;
    add_ffmpeg_output(profile, &mut args)?;

    Ok(CommandPlan {
        executable: "ffmpeg".to_string(),
        args,
    })
}

fn add_ffmpeg_input(profile: &ResolvedProfile, args: &mut Vec<String>) -> Result<()> {
    let size = format!("{}x{}", profile.format.width, profile.format.height);
    let frame_rate = normalize_frame_rate(&profile.format.frame_rate);

    match profile.backend {
        Backend::Avfoundation => {
            args.extend(["-thread_queue_size", "512", "-f", "avfoundation"].map(str::to_string));
            args.extend(["-framerate".to_string(), frame_rate]);
            args.extend(["-video_size".to_string(), size]);
            if let Some(pixel_format) = &profile.format.pixel_format {
                args.extend(["-pixel_format".to_string(), pixel_format.clone()]);
            }
            args.push("-i".to_string());
            args.push(match &profile.audio_device {
                Some(audio_device) => format!("{}:{}", profile.video_device, audio_device),
                None => profile.video_device.clone(),
            });
        }
        Backend::Decklink => {
            let Some(format_code) = &profile.format.decklink_format_code else {
                bail!(
                    "DeckLink profile requires decklink_format_code on format '{}'",
                    profile.format_name
                );
            };
            args.extend(["-thread_queue_size", "512", "-f", "decklink"].map(str::to_string));
            args.extend(["-format_code".to_string(), format_code.clone()]);
            args.extend(["-i".to_string(), profile.video_device.clone()]);
        }
        Backend::V4l2 => {
            args.extend(["-thread_queue_size", "512", "-f", "v4l2"].map(str::to_string));
            args.extend(["-framerate".to_string(), frame_rate]);
            args.extend(["-video_size".to_string(), size]);
            args.extend(["-i".to_string(), profile.video_device.clone()]);
            if let Some(audio_device) = &profile.audio_device {
                args.extend(["-thread_queue_size", "512", "-f", "alsa"].map(str::to_string));
                args.extend(["-i".to_string(), audio_device.clone()]);
            }
        }
        Backend::Lavfi => {
            let sample_rate = profile.format.audio_sample_rate.unwrap_or(48_000);
            args.extend(["-re", "-f", "lavfi", "-i"].map(str::to_string));
            args.push(format!("testsrc2=size={size}:rate={frame_rate}"));
            args.extend(["-f", "lavfi", "-i"].map(str::to_string));
            args.push(format!("sine=frequency=1000:sample_rate={sample_rate}"));
        }
        Backend::GstreamerDecklink | Backend::GstreamerV4l2 | Backend::Aja => {
            unreachable!("handled outside FFmpeg")
        }
    }

    Ok(())
}

fn add_ffmpeg_encoding(profile: &ResolvedProfile, args: &mut Vec<String>) -> Result<()> {
    let bitrate = profile.format.video_bitrate_kbps.unwrap_or(8_000);
    let bitrate_text = format!("{bitrate}k");
    let audio_sample_rate = profile.format.audio_sample_rate.unwrap_or(48_000);
    let audio_channels = profile.format.audio_channels.unwrap_or(2);
    let keyframe_seconds = profile.format.keyframe_seconds.unwrap_or(2.0);
    let fps = frame_rate_as_f64(&profile.format.frame_rate)
        .ok_or_else(|| anyhow::anyhow!("invalid frame rate '{}'", profile.format.frame_rate))?;
    let gop = (fps * f64::from(keyframe_seconds)).round().max(1.0) as u32;

    match profile.encoder {
        VideoEncoder::Copy => {
            args.extend(["-c:v", "copy"].map(str::to_string));
        }
        VideoEncoder::Libx264 => {
            args.extend(
                [
                    "-c:v",
                    "libx264",
                    "-preset",
                    "veryfast",
                    "-tune",
                    "zerolatency",
                ]
                .map(str::to_string),
            );
            args.extend(["-b:v".to_string(), bitrate_text.clone()]);
            args.extend(["-maxrate".to_string(), bitrate_text.clone()]);
            args.extend(["-bufsize".to_string(), format!("{}k", bitrate * 2)]);
            args.extend(["-g".to_string(), gop.to_string()]);
            args.extend(["-pix_fmt", "yuv420p"].map(str::to_string));
        }
        VideoEncoder::H264Videotoolbox => {
            args.extend(["-c:v", "h264_videotoolbox", "-realtime", "true"].map(str::to_string));
            args.extend(["-b:v".to_string(), bitrate_text.clone()]);
            args.extend(["-maxrate".to_string(), bitrate_text]);
            args.extend(["-g".to_string(), gop.to_string()]);
            args.extend(["-pix_fmt", "yuv420p"].map(str::to_string));
        }
        VideoEncoder::H264Nvenc => {
            args.extend(["-c:v", "h264_nvenc", "-preset", "p4", "-tune", "ll"].map(str::to_string));
            args.extend(["-b:v".to_string(), bitrate_text.clone()]);
            args.extend(["-maxrate".to_string(), bitrate_text.clone()]);
            args.extend(["-bufsize".to_string(), format!("{}k", bitrate * 2)]);
            args.extend(["-g".to_string(), gop.to_string()]);
        }
        VideoEncoder::H264Vaapi => {
            args.extend(["-vaapi_device", "/dev/dri/renderD128"].map(str::to_string));
            args.extend(["-vf", "format=nv12,hwupload"].map(str::to_string));
            args.extend(["-c:v", "h264_vaapi"].map(str::to_string));
            args.extend(["-b:v".to_string(), bitrate_text.clone()]);
            args.extend(["-maxrate".to_string(), bitrate_text]);
            args.extend(["-g".to_string(), gop.to_string()]);
        }
    }

    args.extend(["-c:a", "aac"].map(str::to_string));
    args.extend(["-b:a", "192k"].map(str::to_string));
    args.extend(["-ar".to_string(), audio_sample_rate.to_string()]);
    args.extend(["-ac".to_string(), audio_channels.to_string()]);

    Ok(())
}

fn add_ffmpeg_output(profile: &ResolvedProfile, args: &mut Vec<String>) -> Result<()> {
    match profile.output.kind {
        OutputKind::Srt => {
            args.extend(["-f", "mpegts"].map(str::to_string));
            args.push(srt_url_with_options(profile));
        }
        OutputKind::Rtmp => {
            args.extend(["-f", "flv"].map(str::to_string));
            args.push(profile.output.url.clone());
        }
        OutputKind::File => {
            args.push(profile.output.url.clone());
        }
        OutputKind::Whip => bail!(
            "WHIP output is reserved for the RTC hardware-ingest phase; use SRT to stream relay in this version"
        ),
    }
    Ok(())
}

fn build_gstreamer_plan(profile: &ResolvedProfile) -> Result<CommandPlan> {
    if !matches!(profile.output.kind, OutputKind::Srt) {
        bail!("GStreamer runner currently supports SRT output only");
    }

    let width = profile.format.width.to_string();
    let height = profile.format.height.to_string();
    let framerate = gstreamer_frame_rate(&profile.format.frame_rate);
    let bitrate = profile
        .format
        .video_bitrate_kbps
        .unwrap_or(8_000)
        .to_string();
    let keyframe_seconds = profile.format.keyframe_seconds.unwrap_or(2.0);
    let fps = frame_rate_as_f64(&profile.format.frame_rate)
        .ok_or_else(|| anyhow::anyhow!("invalid frame rate '{}'", profile.format.frame_rate))?;
    let key_int_max = (fps * f64::from(keyframe_seconds))
        .round()
        .max(1.0)
        .to_string();
    let srt_url = srt_url_with_options(profile);

    let mut args = vec!["-e".to_string()];
    match profile.backend {
        Backend::GstreamerDecklink => {
            args.extend([
                "decklinkvideosrc".to_string(),
                format!("device-number={}", profile.video_device),
                "!".to_string(),
                "videoconvert".to_string(),
                "!".to_string(),
            ]);
        }
        Backend::GstreamerV4l2 => {
            args.extend([
                "v4l2src".to_string(),
                format!("device={}", profile.video_device),
                "!".to_string(),
                format!("video/x-raw,width={width},height={height},framerate={framerate}"),
                "!".to_string(),
                "videoconvert".to_string(),
                "!".to_string(),
            ]);
        }
        _ => unreachable!("only gstreamer backends are handled here"),
    }

    args.extend([
        "x264enc".to_string(),
        "tune=zerolatency".to_string(),
        "speed-preset=veryfast".to_string(),
        format!("bitrate={bitrate}"),
        format!("key-int-max={key_int_max}"),
        "!".to_string(),
        "h264parse".to_string(),
        "config-interval=1".to_string(),
        "!".to_string(),
        "mpegtsmux".to_string(),
        "name=mux".to_string(),
        "!".to_string(),
        "srtsink".to_string(),
        format!("uri={srt_url}"),
    ]);

    if let Some(audio_device) = &profile.audio_device {
        args.extend([
            "alsasrc".to_string(),
            format!("device={audio_device}"),
            "!".to_string(),
            "audioconvert".to_string(),
            "!".to_string(),
            "audioresample".to_string(),
            "!".to_string(),
            "avenc_aac".to_string(),
            "bitrate=192000".to_string(),
            "!".to_string(),
            "aacparse".to_string(),
            "!".to_string(),
            "mux.".to_string(),
        ]);
    }

    Ok(CommandPlan {
        executable: "gst-launch-1.0".to_string(),
        args,
    })
}

fn srt_url_with_options(profile: &ResolvedProfile) -> String {
    let mut url = profile.output.url.clone();
    let latency_us = profile.output.srt_latency_ms.unwrap_or(120) * 1_000;
    let mode = profile.output.srt_mode.as_deref().unwrap_or("caller");
    append_query_pair(&mut url, "mode", mode);
    append_query_pair(&mut url, "transtype", "live");
    append_query_pair(&mut url, "latency", &latency_us.to_string());
    url
}

fn append_query_pair(url: &mut String, key: &str, value: &str) {
    if url.contains('?') {
        if !url.ends_with('?') && !url.ends_with('&') {
            url.push('&');
        }
    } else {
        url.push('?');
    }
    url.push_str(key);
    url.push('=');
    url.push_str(value);
}

pub fn normalize_frame_rate(value: &str) -> String {
    match value.trim() {
        "23.976" | "23.98" => "24000/1001".to_string(),
        "29.97" => "30000/1001".to_string(),
        "59.94" => "60000/1001".to_string(),
        other => other.to_string(),
    }
}

pub fn frame_rate_as_f64(value: &str) -> Option<f64> {
    match value.trim() {
        "24000/1001" | "23.976" | "23.98" => Some(23.976),
        "30000/1001" | "29.97" => Some(29.97),
        "60000/1001" | "59.94" => Some(59.94),
        other => other.parse::<f64>().ok(),
    }
}

fn gstreamer_frame_rate(value: &str) -> String {
    match value.trim() {
        "23.976" | "23.98" => "24000/1001".to_string(),
        "29.97" => "30000/1001".to_string(),
        "59.94" => "60000/1001".to_string(),
        other => format!("{other}/1"),
    }
}

#[cfg(test)]
mod tests {
    use capture_common::{FormatConfig, OutputConfig, ScanMode};

    use super::*;

    fn test_profile() -> ResolvedProfile {
        ResolvedProfile {
            name: "test".to_string(),
            purpose: Some("program".to_string()),
            backend: Backend::Lavfi,
            video_device: "testsrc2".to_string(),
            audio_device: None,
            format_name: "1080p29.97".to_string(),
            format: FormatConfig {
                width: 1920,
                height: 1080,
                frame_rate: "29.97".to_string(),
                scan: ScanMode::Progressive,
                pixel_format: None,
                decklink_format_code: Some("Hp29".to_string()),
                video_bitrate_kbps: Some(8_000),
                keyframe_seconds: Some(2.0),
                audio_sample_rate: Some(48_000),
                audio_channels: Some(2),
            },
            output_name: "local".to_string(),
            output: OutputConfig {
                kind: OutputKind::Srt,
                url: "srt://localhost:9999?streamid=default/broadcast/test".to_string(),
                srt_latency_ms: Some(120),
                srt_mode: Some("caller".to_string()),
            },
            encoder: VideoEncoder::Libx264,
            safe_mode: false,
        }
    }

    #[test]
    fn builds_srt_plan_with_fractional_rate() {
        let plan = build_plan(&test_profile()).unwrap();
        let line = plan.shell_line();
        assert!(line.contains("testsrc2=size=1920x1080:rate=30000/1001"));
        assert!(line.contains("srt://localhost:9999?streamid=default/broadcast/test&mode=caller&transtype=live&latency=120000"));
    }

    #[test]
    fn builds_gstreamer_v4l2_plan() {
        let mut profile = test_profile();
        profile.backend = Backend::GstreamerV4l2;
        profile.video_device = "/dev/video0".to_string();
        let plan = build_plan(&profile).unwrap();
        assert_eq!(plan.executable, "gst-launch-1.0");
        assert!(plan.shell_line().contains("v4l2src"));
    }
}
