use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use capture_common::CommandPlan;

pub fn run_plan_blocking(plan: &CommandPlan) -> Result<()> {
    which::which(&plan.executable)
        .with_context(|| format!("{} is required but was not found on PATH", plan.executable))?;

    let status = Command::new(&plan.executable)
        .args(&plan.args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("failed to launch {}", plan.executable))?;

    if !status.success() {
        bail!("capture bridge exited with status {status}");
    }

    Ok(())
}
