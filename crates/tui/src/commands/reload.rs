//! `/reload` — rebuild, install, and restart the TUI.

use super::CommandResult;

/// Rebuild the TUI binary and install it to `~/.cargo/bin/deepseek`.
///
/// Spawns a background build + install, then tells the user to restart.
/// The build runs asynchronously so the TUI stays responsive; the user
/// can continue working while it compiles.
pub fn reload() -> CommandResult {
    let Some(source_dir) = dirs_sys() else {
        return CommandResult::error("Cannot find source directory (set DEEPSEEK_SOURCE_DIR)");
    };

    // Spawn a background shell that builds and installs.
    let script = format!(
        "cd {} && cargo build --release -p deepseek-tui 2>&1 && cp target/release/deepseek-tui {} && echo '✓ reload ready — restart deepseek'",
        shell_escape(&source_dir),
        shell_escape(&install_path()),
    );

    match std::process::Command::new("sh")
        .arg("-c")
        .arg(&script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_child) => {
            CommandResult::message(format!(
                "Building deepseek-tui in background…\n\
                 Check progress in another terminal:\n  cargo build --release\n\
                 Once done:\n  cp {src}/target/release/deepseek-tui {dst}\n  deepseek resume latest",
                src = source_dir,
                dst = install_path(),
            ))
        }
        Err(e) => CommandResult::error(format!("Failed to start build: {e}")),
    }
}

fn install_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/vesper".to_string());
    format!("{home}/.cargo/bin/deepseek")
}

fn dirs_sys() -> Option<String> {
    // Use a known workspace path. In practice this is ~/ds_cfg/source.
    let candidates = [
        std::env::var("DEEPSEEK_SOURCE_DIR").ok(),
        std::env::current_dir()
            .ok()
            .map(|p| p.to_string_lossy().to_string()),
        Some("/Users/vesper/ds_cfg/source".to_string()),
    ];
    for c in candidates.iter().flatten() {
        let p = std::path::Path::new(c);
        if p.join("Cargo.toml").exists() {
            return Some(c.clone());
        }
    }
    None
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
