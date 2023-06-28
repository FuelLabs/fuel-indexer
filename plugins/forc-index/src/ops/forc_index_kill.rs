use crate::cli::KillCommand;
use std::process::Command;
use tracing::info;

pub fn kill(command: KillCommand) -> anyhow::Result<()> {
    let port_number = command.port.parse::<u16>().unwrap();

    kill_process_by_port(port_number, command.kill)?;

    Ok(())
}

fn kill_process_by_port(port: u16, kill: bool) -> anyhow::Result<()> {
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

    let signal = if kill { "kill" } else { "terminate" };

    let mut cmd = Command::new("kill");
    if kill {
        cmd.arg("-9");
    }
    cmd.arg(pid.to_string())
        .status()
        .map_err(|e| anyhow::anyhow!("❌ Failed to {signal} process: {}", e))?;

    let signal = if kill { "killed" } else { "terminated" };
    info!("✅ Sucessfully {signal} process {pid} listening on port {port}");

    Ok(())
}
