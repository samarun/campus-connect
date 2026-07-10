use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use capture_common::{
    CaptureConfig, PlatformProfileOverrides, ProfileConfig, ProfileOverrides, ResolvedProfile,
};

pub fn load_config(path: &Path) -> Result<CaptureConfig> {
    let resolved_path = resolve_config_path(path);
    let raw = fs::read_to_string(&resolved_path).with_context(|| {
        format!(
            "unable to read capture bridge config {}",
            resolved_path.display()
        )
    })?;
    let config = toml::from_str::<CaptureConfig>(&raw).with_context(|| {
        format!(
            "unable to parse capture bridge config {}",
            resolved_path.display()
        )
    })?;
    Ok(config)
}

pub fn save_profile(path: &Path, name: &str, profile: ProfileConfig) -> Result<CaptureConfig> {
    let resolved_path = resolve_config_path(path);
    let mut config = load_config(&resolved_path)?;
    config.profiles.insert(name.to_string(), profile);
    let raw = toml::to_string_pretty(&config).context("unable to serialize capture config")?;
    fs::write(&resolved_path, raw)
        .with_context(|| format!("unable to write capture config {}", resolved_path.display()))?;
    Ok(config)
}

pub fn resolve_profile(
    config: &CaptureConfig,
    profile_name: &str,
    overrides: ProfileOverrides,
) -> Result<ResolvedProfile> {
    let profile = config
        .profiles
        .get(profile_name)
        .with_context(|| format!("profile '{profile_name}' does not exist"))?;
    let platform = active_platform_overrides(profile);

    let backend = overrides
        .backend
        .or(platform.and_then(|value| value.backend))
        .or(profile.backend)
        .or(config.defaults.backend)
        .unwrap_or_default();
    let video_device = overrides
        .video_device
        .or_else(|| platform.and_then(|value| value.video_device.clone()))
        .or_else(|| profile.video_device.clone())
        .or_else(|| config.defaults.video_device.clone())
        .with_context(|| format!("profile '{profile_name}' is missing video_device"))?;
    let audio_device = overrides
        .audio_device
        .or_else(|| platform.and_then(|value| value.audio_device.clone()))
        .or_else(|| profile.audio_device.clone())
        .or_else(|| config.defaults.audio_device.clone());
    let format_name = overrides
        .format
        .or_else(|| platform.and_then(|value| value.format.clone()))
        .or_else(|| profile.format.clone())
        .or_else(|| config.defaults.format.clone())
        .with_context(|| format!("profile '{profile_name}' is missing format"))?;
    let output_name = overrides
        .output
        .or_else(|| platform.and_then(|value| value.output.clone()))
        .or_else(|| profile.output.clone())
        .or_else(|| config.defaults.output.clone())
        .with_context(|| format!("profile '{profile_name}' is missing output"))?;
    let encoder = overrides
        .encoder
        .or(platform.and_then(|value| value.encoder))
        .or(profile.encoder)
        .or(config.defaults.encoder)
        .unwrap_or_default();

    let mut format = config
        .formats
        .get(&format_name)
        .cloned()
        .with_context(|| format!("format '{format_name}' does not exist"))?;
    if let Some(frame_rate) = overrides.frame_rate {
        format.frame_rate = frame_rate;
    }
    if let Some(video_bitrate_kbps) = overrides.video_bitrate_kbps {
        if video_bitrate_kbps == 0 {
            bail!("video bitrate must be greater than zero");
        }
        format.video_bitrate_kbps = Some(video_bitrate_kbps);
    }

    let output = config
        .outputs
        .get(&output_name)
        .cloned()
        .with_context(|| format!("output '{output_name}' does not exist"))?;

    Ok(ResolvedProfile {
        name: profile_name.to_string(),
        purpose: profile.purpose.clone(),
        backend,
        video_device,
        audio_device,
        format_name,
        format,
        output_name,
        output,
        encoder,
        safe_mode: profile.safe_mode.unwrap_or(false),
    })
}

fn resolve_config_path(path: &Path) -> std::path::PathBuf {
    if path.exists() {
        return path.to_path_buf();
    }

    let repo_root_path = Path::new("capture-bridge").join(path);
    if repo_root_path.exists() {
        repo_root_path
    } else {
        path.to_path_buf()
    }
}

fn active_platform_overrides(profile: &ProfileConfig) -> Option<&PlatformProfileOverrides> {
    if cfg!(target_os = "macos") {
        profile.macos.as_ref()
    } else if cfg!(target_os = "linux") {
        profile.linux.as_ref()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use capture_common::{Backend, VideoEncoder};

    use super::*;

    #[test]
    fn resolves_overrides() {
        let raw = r#"
            [defaults]
            backend = "lavfi"
            video_device = "test"
            format = "1080p30"
            output = "local"
            encoder = "libx264"

            [formats.1080p30]
            width = 1920
            height = 1080
            frame_rate = "30"

            [outputs.local]
            kind = "srt"
            url = "srt://localhost:9999?streamid=default/broadcast/test"

            [profiles.test]
            purpose = "test"
        "#;
        let config = toml::from_str::<CaptureConfig>(raw).unwrap();
        let resolved = resolve_profile(
            &config,
            "test",
            ProfileOverrides {
                frame_rate: Some("29.97".to_string()),
                encoder: Some(VideoEncoder::H264Videotoolbox),
                ..ProfileOverrides::default()
            },
        )
        .unwrap();

        assert_eq!(resolved.backend, Backend::Lavfi);
        assert_eq!(resolved.format.frame_rate, "29.97");
        assert_eq!(resolved.encoder, VideoEncoder::H264Videotoolbox);
    }
}
