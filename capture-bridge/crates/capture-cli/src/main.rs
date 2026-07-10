use std::path::PathBuf;

use anyhow::Result;
use capture_common::{Backend, ProfileOverrides, VideoEncoder};
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "capture-bridge",
    version,
    about = "Campus Connect Capture Bridge for SDI/HDMI capture, format selection, and stream relay/RTC handoff"
)]
struct Cli {
    #[arg(
        short,
        long,
        global = true,
        default_value = "config/capture.local.toml"
    )]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List capture devices exposed by the selected backend.
    ListDevices {
        #[arg(default_value_t = Backend::Avfoundation)]
        backend: Backend,
    },
    /// List built-in recommended broadcast formats.
    ListFormats,
    /// Validate one profile or every profile in the config.
    Validate {
        #[arg(short, long)]
        profile: Option<String>,
    },
    /// Run production preflight checks for a profile.
    Preflight(ProfileCommand),
    /// Print the exact media command for a profile.
    Plan(ProfileCommand),
    /// Run a profile. Use --dry-run first in production.
    Run(RunCommand),
}

#[derive(Args, Debug)]
struct ProfileCommand {
    #[arg(short, long, default_value = "program-local-srt")]
    profile: String,

    #[arg(long)]
    json: bool,

    #[command(flatten)]
    overrides: OverrideArgs,
}

#[derive(Args, Debug)]
struct RunCommand {
    #[arg(short, long, default_value = "program-local-srt")]
    profile: String,

    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    overrides: OverrideArgs,
}

#[derive(Args, Debug, Default, Clone)]
struct OverrideArgs {
    /// Override backend: avfoundation, decklink, v4l2, lavfi, gstreamer-decklink, gstreamer-v4l2, aja.
    #[arg(long)]
    backend: Option<Backend>,

    /// Override video device. Examples: 1, /dev/video0, "DeckLink Duo (1)".
    #[arg(long)]
    video_device: Option<String>,

    /// Override audio device. Examples: 2, hw:0, embedded.
    #[arg(long)]
    audio_device: Option<String>,

    /// Override format preset. Examples: 1080p30, 1080p29.97, 720p60.
    #[arg(long)]
    format: Option<String>,

    /// Override frame rate inside the selected format. Examples: 25, 29.97, 30, 50, 59.94.
    #[arg(long)]
    frame_rate: Option<String>,

    /// Override output target from the config. Examples: local-program, production-program.
    #[arg(long)]
    output: Option<String>,

    /// Override video encoder. Examples: libx264, h264-videotoolbox, h264-nvenc, h264-vaapi, copy.
    #[arg(long)]
    encoder: Option<VideoEncoder>,

    /// Override video bitrate in kbps.
    #[arg(long)]
    video_bitrate_kbps: Option<u32>,
}

impl From<OverrideArgs> for ProfileOverrides {
    fn from(value: OverrideArgs) -> Self {
        Self {
            backend: value.backend,
            video_device: value.video_device,
            audio_device: value.audio_device,
            format: value.format,
            frame_rate: value.frame_rate,
            output: value.output,
            encoder: value.encoder,
            video_bitrate_kbps: value.video_bitrate_kbps,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListDevices { backend } => {
            let output = capture_core::list_devices(backend)?;
            print!("{output}");
        }
        Commands::ListFormats => {
            println!("{}", capture_core::recommended_formats_table());
        }
        Commands::Validate { profile } => {
            let capture_config = capture_core::load_config(&cli.config)?;
            let profile_names: Vec<String> = match profile {
                Some(name) => vec![name],
                None => capture_config.profiles.keys().cloned().collect(),
            };

            for profile_name in profile_names {
                let resolved = capture_core::resolve_profile(
                    &capture_config,
                    &profile_name,
                    ProfileOverrides::default(),
                )?;
                let warnings = capture_core::validate_profile(&resolved);
                println!("OK profile: {profile_name}");
                for warning in warnings {
                    println!("  warning: {warning}");
                }
            }
        }
        Commands::Preflight(command) => {
            let capture_config = capture_core::load_config(&cli.config)?;
            let resolved = capture_core::resolve_profile(
                &capture_config,
                &command.profile,
                command.overrides.into(),
            )?;
            let warnings = capture_core::validate_profile(&resolved);
            let checks = capture_core::preflight_profile(&resolved);

            if command.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&capture_common::ValidationReport {
                        profile: command.profile,
                        warnings,
                        checks,
                    })?
                );
            } else {
                for warning in warnings {
                    println!("warning: {warning}");
                }
                for check in checks {
                    println!("{:?}: {} - {}", check.level, check.name, check.message);
                }
            }
        }
        Commands::Plan(command) => {
            let capture_config = capture_core::load_config(&cli.config)?;
            let resolved = capture_core::resolve_profile(
                &capture_config,
                &command.profile,
                command.overrides.into(),
            )?;
            let plan = capture_core::build_plan(&resolved)?;

            if command.json {
                println!("{}", serde_json::to_string_pretty(&plan)?);
            } else {
                for warning in capture_core::validate_profile(&resolved) {
                    println!("warning: {warning}");
                }
                println!("{}", plan.shell_line());
            }
        }
        Commands::Run(command) => {
            let capture_config = capture_core::load_config(&cli.config)?;
            let resolved = capture_core::resolve_profile(
                &capture_config,
                &command.profile,
                command.overrides.into(),
            )?;
            let plan = capture_core::build_plan(&resolved)?;

            for warning in capture_core::validate_profile(&resolved) {
                println!("warning: {warning}");
            }
            println!("{}", plan.shell_line());

            if !command.dry_run {
                capture_core::runner::run_plan_blocking(&plan)?;
            }
        }
    }

    Ok(())
}
