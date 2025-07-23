use super::command_translation::translate_command_for_windows;
use std::collections::HashSet;
use std::process::{Command, Stdio};
use std::time::Duration;

// Command output structure
pub struct CommandOutput {
  pub stdout: String,
  pub stderr: String,
  pub status: std::process::ExitStatus,
}

// Secure command structure for validated commands
#[derive(Debug)]
pub struct SecureCommand {
  pub program: String,
  pub args: Vec<String>,
}

// Parse and validate command using whitelist approach
pub fn parse_secure_command(cmd: &str) -> Result<SecureCommand, String> {
  let cmd = cmd.trim();
  if cmd.is_empty() {
    return Err("Empty command".to_string());
  }

  // On Windows, translate Unix commands to PowerShell
  #[cfg(target_os = "windows")]
  let cmd_string = translate_command_for_windows(cmd);
  #[cfg(target_os = "windows")]
  let cmd_to_parse = cmd_string.as_str();
  #[cfg(target_os = "windows")]
  let is_powershell_command = {
    // Check if the translation resulted in a PowerShell command
    cmd_string.contains("Get-")
      || cmd_string.contains("Select-")
      || cmd_string.contains("Format-")
      || cmd_string.contains(" | ")
  };

  // For non-Windows, keep the original reference
  #[cfg(not(target_os = "windows"))]
  let cmd_to_parse = cmd;
  #[cfg(not(target_os = "windows"))]
  let is_powershell_command = false;

  // Whitelist of allowed commands - focus on read-only, generally safe commands
  // Security Note: Even read-only commands can have security implications:
  // - Some may read sensitive files if given appropriate paths
  // - Network commands (curl, wget) can make outbound connections
  // - Archive commands may extract to arbitrary locations with specially
  //   crafted files
  // However, these are standard system utilities and the risk is acceptable for
  // a text reader
  let allowed_commands: HashSet<&str> = [
    // File/directory listing and navigation
    "ls",
    "pwd",
    "find",
    "locate",
    "which",
    "whereis",
    // File viewing and reading (core functionality for text reader)
    "cat",
    "less",
    "more",
    "head",
    "tail",
    "file",
    "stat",
    "wc",
    "nl",
    // Text processing (read-only operations)
    "grep",
    "awk",
    "sed",
    "sort",
    "uniq",
    "cut",
    "tr",
    "fmt",
    "fold",
    // System information (generally safe, read-only)
    "date",
    "uptime",
    "whoami",
    "id",
    "uname",
    "hostname",
    "df",
    "free",
    "ps",
    "top",
    "env",
    "printenv",
    "history",
    // Archive viewing (read-only access, but see security note above)
    "tar",
    "zip",
    "unzip",
    "gzip",
    "gunzip",
    "zcat",
    // Network utilities (outbound connections only, read-only data)
    "ping",
    "dig",
    "nslookup",
    "curl",
    "wget",
    // Text utilities (path manipulation, generally safe)
    "echo",
    "printf",
    "basename",
    "dirname",
    "realpath",
    "readlink",
    // PowerShell commands (Windows)
    "Get-ChildItem",
    "Get-Content",
    "Get-Location",
    "Select-String",
    "Get-Date",
    "Get-Process",
    "Get-Host",
    "Format-Table",
    "Select-Object",
    "Measure-Object",
    "Where-Object",
    "Sort-Object",
    // PowerShell.exe for Windows
    "powershell.exe",
    "powershell",
  ]
  .iter()
  .cloned()
  .collect();

  // Split command into parts
  let parts: Vec<&str> = cmd_to_parse.split_whitespace().collect();
  if parts.is_empty() {
    return Err("Invalid command".to_string());
  }

  let program = parts[0];

  // Check if command is whitelisted
  if !allowed_commands.contains(program) {
    return Err(format!("Command '{program}' is not allowed"));
  }

  // Validate arguments - reject dangerous characters to prevent shell injection
  // Even though we're not using shell execution, some commands might interpret
  // these
  let dangerous_chars: &[char] = if is_powershell_command {
    // For PowerShell commands, allow pipes but still restrict other dangerous
    // chars
    &['&', ';', '`', '$', '(', ')', '<', '>', '\\', '*', '?']
  } else {
    // For regular commands, maintain strict validation
    &['|', '&', ';', '`', '$', '(', ')', '<', '>', '\\', '*', '?']
  };

  // Special handling for PowerShell - don't validate the full command string
  if is_powershell_command {
    // For PowerShell, we'll validate differently since the whole command is one
    // string Just check for the most dangerous characters
    if cmd_to_parse.chars().any(|c| ['`', ';', '&'].contains(&c)) {
      return Err(
        "PowerShell command contains dangerous characters".to_string(),
      );
    }
  } else {
    // Regular validation for non-PowerShell commands
    for arg in &parts[1..] {
      if arg.chars().any(|c| dangerous_chars.contains(&c)) {
        return Err(format!("Argument contains dangerous characters: {arg}"));
      }

      // Additional safety: reject very long arguments that could cause buffer
      // overflows
      if arg.len() > 1000 {
        return Err("Argument too long (max 1000 characters)".to_string());
      }
    }
  }

  // Additional safety: limit total number of arguments
  if parts.len() > 50 {
    return Err("Too many arguments (max 50)".to_string());
  }

  // On Windows, if we have a PowerShell command, wrap it properly
  #[cfg(target_os = "windows")]
  {
    // Check if this is a PowerShell cmdlet or contains pipes
    if is_powershell_command
      || program.contains('-')
      || program.starts_with("Get-")
      || program.starts_with("Select-")
    {
      // For PowerShell commands, pass the entire translated command as a single
      // argument
      return Ok(SecureCommand {
        program: "powershell.exe".to_string(),
        args: vec![
          "-NoProfile".to_string(),
          "-Command".to_string(),
          cmd_to_parse.to_string(),
        ],
      });
    }
  }

  Ok(SecureCommand {
    program: program.to_string(),
    args: parts[1..].iter().map(|s| s.to_string()).collect(),
  })
}

// Execute a validated command with timeout
pub fn execute_secure_command_with_timeout(
  secure_cmd: SecureCommand,
  timeout: Duration,
) -> Result<CommandOutput, String> {
  let mut child = Command::new(&secure_cmd.program)
    .args(&secure_cmd.args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|e| {
      format!("Failed to execute command '{}': {}", secure_cmd.program, e)
    })?;

  // Wait for the command with timeout
  match child.wait_timeout(timeout) {
    Ok(Some(status)) => {
      // Command completed within timeout
      let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to read output: {e}"))?;

      Ok(CommandOutput {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status,
      })
    }
    Ok(None) => {
      // Timeout occurred, kill the process
      let _ = child.kill();
      Err("Command timed out after 30 seconds".to_string())
    }
    Err(e) => Err(format!("Failed to wait for command: {e}")),
  }
}

// Extension trait for waiting with timeout
trait WaitTimeout {
  fn wait_timeout(
    &mut self,
    dur: Duration,
  ) -> std::io::Result<Option<std::process::ExitStatus>>;
}

impl WaitTimeout for std::process::Child {
  fn wait_timeout(
    &mut self,
    dur: Duration,
  ) -> std::io::Result<Option<std::process::ExitStatus>> {
    let start = std::time::Instant::now();

    loop {
      match self.try_wait()? {
        Some(status) => return Ok(Some(status)),
        None => {
          if start.elapsed() >= dur {
            return Ok(None);
          }
          std::thread::sleep(Duration::from_millis(100));
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::parse_secure_command;

  #[test]
  fn test_allowed_commands() {
    let allowed = vec!["cat", "less", "head", "tail", "grep", "ls", "pwd"];
    for cmd in allowed {
      assert!(parse_secure_command(cmd).is_ok(), "{cmd} should be allowed");
    }
  }

  #[test]
  fn test_rejected_commands() {
    let rejected = vec!["rm", "sudo", "kill", "reboot"];
    for cmd in rejected {
      assert!(parse_secure_command(cmd).is_err(), "{cmd} should be rejected");
    }
  }

  #[test]
  fn test_dangerous_chars() {
    let dangerous =
      vec!["cat file; rm file", "echo `cmd`", "ls > file", "cmd | other"];
    for input in dangerous {
      assert!(
        parse_secure_command(input).is_err(),
        "{input} should be rejected"
      );
    }
  }
}
