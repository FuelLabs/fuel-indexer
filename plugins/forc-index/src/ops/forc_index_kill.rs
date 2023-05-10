use crate::cli::KillCommand;
use std::process::Command;
use tracing::info;

pub fn kill(command: KillCommand) -> anyhow::Result<()> {
    let port_number = command.port.parse::<u16>().unwrap();

    kill_process_by_port(port_number)?;

    Ok(())
}

fn kill_process_by_port(port: u16) -> anyhow::Result<()> {
    let output = Command::new("lsof")
        .arg("-ti")
        .arg(format!(":{}", port))
        .output()?;

    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if pid_str.is_empty() {
        return Err(anyhow::anyhow!(
            "❌ No process is listening on port {}",
            port
        ));
    }

    let pid = pid_str
        .parse::<i32>()
        .map_err(|e| anyhow::anyhow!("❌ Failed to parse PID: {}", e))?;

    Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status()
        .map_err(|e| anyhow::anyhow!("❌ Failed to kill process: {}", e))?;

    info!(
        "✅ Sucessfully killed process {} listening on port {}",
        pid, port
    );

    Ok(())
}
