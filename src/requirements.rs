use crate::{InstallSource, REQUIREMENTS_ERR_CODEX_MISSING, format_cmd};

pub fn ensure_codex_cli(source: InstallSource) -> Result<(), String> {
    ensure_codex_cli_with(source, cfg!(debug_assertions))
}

fn ensure_codex_cli_with(source: InstallSource, is_debug: bool) -> Result<(), String> {
    if is_debug {
        return Ok(());
    }
    if command_exists("codex") {
        return Ok(());
    }
    let install_cmd = format_cmd(&install_command(source), false);
    Err(crate::msg1(REQUIREMENTS_ERR_CODEX_MISSING, install_cmd))
}

fn install_command(source: InstallSource) -> String {
    match source {
        InstallSource::Npm => "npm install -g @openai/codex".to_string(),
        InstallSource::Bun => "bun install -g @openai/codex".to_string(),
        InstallSource::Brew => "brew install --cask codex".to_string(),
        InstallSource::Unknown => platform_default_install_command(),
    }
}

fn command_exists(command: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    let candidates = command_candidates(command);
    for dir in std::env::split_paths(&path) {
        for candidate in &candidates {
            if dir.join(candidate).is_file() {
                return true;
            }
        }
    }
    false
}

#[cfg(windows)]
fn command_candidates(command: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let path = std::path::Path::new(command);
    if path.extension().is_some() {
        candidates.push(command.to_string());
        return candidates;
    }
    let pathext = std::env::var_os("PATHEXT")
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| ".EXE;.CMD;.BAT;.COM".to_string());
    for ext in pathext.split(';').filter(|ext| !ext.is_empty()) {
        candidates.push(format!("{command}{ext}"));
    }
    candidates
}

#[cfg(not(windows))]
fn command_candidates(command: &str) -> Vec<String> {
    vec![command.to_string()]
}

#[cfg(windows)]
fn platform_default_install_command() -> String {
    "winget install OpenAI.Codex".to_string()
}

#[cfg(target_os = "macos")]
fn platform_default_install_command() -> String {
    "brew install --cask codex or npm install -g @openai/codex".to_string()
}

#[cfg(all(not(target_os = "macos"), not(windows)))]
fn platform_default_install_command() -> String {
    "npm install -g @openai/codex".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{ENV_MUTEX, set_env_guard};
    use std::fs;

    #[test]
    fn ensure_codex_cli_skips_in_debug() {
        ensure_codex_cli_with(InstallSource::Npm, true).unwrap();
    }

    #[test]
    fn ensure_codex_cli_passes_when_codex_exists() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = dir.path().join("codex");
        fs::write(&bin, "stub").expect("write bin");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&bin).expect("meta").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&bin, perms).expect("chmod");
        }
        let path = dir.path().to_string_lossy().into_owned();
        let _env = set_env_guard("PATH", Some(&path));
        ensure_codex_cli_with(InstallSource::Npm, false).unwrap();
    }

    #[test]
    fn ensure_codex_cli_errors_when_missing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let _env = set_env_guard("PATH", Some(""));
        let err = ensure_codex_cli_with(InstallSource::Npm, false).unwrap_err();
        assert!(err.contains("codex"));
    }

    #[test]
    fn install_commands_cover_sources() {
        assert!(install_command(InstallSource::Npm).contains("npm"));
        assert!(install_command(InstallSource::Bun).contains("bun"));
        assert!(install_command(InstallSource::Brew).contains("brew"));
        assert!(!platform_default_install_command().is_empty());
    }

    #[test]
    fn command_candidates_non_windows() {
        let candidates = command_candidates("codex");
        assert_eq!(candidates, vec!["codex".to_string()]);
    }
}
