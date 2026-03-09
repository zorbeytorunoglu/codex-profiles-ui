use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    InstallSource, Paths, current_saved_id, detect_install_source, is_profile_ready, lock_usage,
    print_output_block, profile_id_from_path, read_auth_file, read_tokens,
    repair_profiles_metadata,
};

#[derive(Clone, Copy, Debug, Default)]
enum Level {
    Ok,
    Warn,
    Error,
    #[default]
    Info,
}

impl Level {
    fn label(self) -> &'static str {
        match self {
            Level::Ok => "ok",
            Level::Warn => "warn",
            Level::Error => "error",
            Level::Info => "info",
        }
    }
}

#[derive(Default, Serialize)]
struct Counts {
    ok: usize,
    warn: usize,
    error: usize,
    info: usize,
}

impl Counts {
    fn add(&mut self, level: Level) {
        match level {
            Level::Ok => self.ok += 1,
            Level::Warn => self.warn += 1,
            Level::Error => self.error += 1,
            Level::Info => self.info += 1,
        }
    }
}

struct Check {
    level: Level,
    name: &'static str,
    detail: String,
}

#[derive(Serialize)]
struct DoctorCheckJson {
    name: &'static str,
    level: &'static str,
    detail: String,
}

#[derive(Serialize)]
struct DoctorJson {
    checks: Vec<DoctorCheckJson>,
    summary: Counts,
    #[serde(skip_serializing_if = "Option::is_none")]
    repairs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl Check {
    fn new(level: Level, name: &'static str, detail: impl Into<String>) -> Self {
        Self {
            level,
            name,
            detail: detail.into(),
        }
    }

    fn render(&self) -> String {
        format!("[{}] {}: {}", self.level.label(), self.name, self.detail)
    }
}

enum AuthState {
    Missing,
    Valid,
    Incomplete(String),
    Invalid(String),
}

struct SavedProfilesReport {
    check: Check,
    tokens: BTreeMap<String, Result<crate::Tokens, String>>,
}

pub fn doctor(paths: &Paths, fix: bool, json: bool) -> Result<(), String> {
    let repairs = if fix {
        match repair(paths) {
            Ok(repairs) => Some(repairs),
            Err(err) => {
                if json {
                    let checks = collect_checks(paths);
                    let counts = summarize_checks(&checks);
                    return print_doctor_json(checks, counts, Some(Vec::new()), Some(err));
                }
                return Err(err);
            }
        }
    } else {
        None
    };
    let checks = collect_checks(paths);
    let counts = summarize_checks(&checks);

    if json {
        return print_doctor_json(checks, counts, repairs, None);
    }

    let mut lines = vec!["Doctor".to_string(), String::new()];

    for check in checks {
        lines.push(check.render());
    }

    lines.push(String::new());
    lines.push(format!(
        "Summary: {} ok, {} warn, {} error, {} info",
        counts.ok, counts.warn, counts.error, counts.info
    ));
    if let Some(repairs) = repairs {
        lines.push(String::new());
        if repairs.is_empty() {
            lines.push("No repairs needed.".to_string());
        } else {
            lines.push("Repairs applied:".to_string());
            for repair in repairs {
                lines.push(format!("- {repair}"));
            }
        }
    }
    print_output_block(&lines.join("\n"));
    Ok(())
}

fn repair(paths: &Paths) -> Result<Vec<String>, String> {
    let mut repairs = repair_storage(paths)?;
    repairs.extend(repair_profiles_metadata(paths)?);
    Ok(repairs)
}

fn repair_storage(paths: &Paths) -> Result<Vec<String>, String> {
    let mut repairs = Vec::new();

    if paths.profiles.exists() {
        if !paths.profiles.is_dir() {
            return Err("Error: profiles directory exists but is not a directory".to_string());
        }
    } else {
        fs::create_dir_all(&paths.profiles).map_err(|err| err.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&paths.profiles, fs::Permissions::from_mode(0o700))
                .map_err(|err| err.to_string())?;
        }
        repairs.push("Created profiles directory".to_string());
    }

    if paths.profiles_index.exists() && !paths.profiles_index.is_file() {
        return Err("Error: profiles index exists but is not a file".to_string());
    }

    if paths.profiles_lock.exists() {
        if !paths.profiles_lock.is_file() {
            return Err("Error: profiles lock exists but is not a file".to_string());
        }
    } else {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&paths.profiles_lock)
            .map_err(|err| err.to_string())?;
        repairs.push("Created profiles lock file".to_string());
    }

    Ok(repairs)
}

fn collect_checks(paths: &Paths) -> Vec<Check> {
    let mut checks: Vec<Check> = inspect_install(paths)
        .into_iter()
        .chain(inspect_auth(paths))
        .chain(inspect_profiles_dir(paths))
        .chain(inspect_profiles_index(paths))
        .chain(inspect_profiles_lock(paths))
        .collect();

    let saved = inspect_saved_profiles(paths);
    checks.push(saved.check);
    checks.push(inspect_current_profile(paths, &saved.tokens));
    checks
}

fn summarize_checks(checks: &[Check]) -> Counts {
    let mut counts = Counts::default();
    for check in checks {
        counts.add(check.level);
    }
    counts
}

fn print_doctor_json(
    checks: Vec<Check>,
    summary: Counts,
    repairs: Option<Vec<String>>,
    error: Option<String>,
) -> Result<(), String> {
    let payload = DoctorJson {
        checks: checks
            .into_iter()
            .map(|check| DoctorCheckJson {
                name: check.name,
                level: check.level.label(),
                detail: check.detail,
            })
            .collect(),
        summary,
        repairs,
        error,
    };
    let json = serde_json::to_string_pretty(&payload).map_err(|err| err.to_string())?;
    println!("{json}");
    Ok(())
}

fn inspect_install(_paths: &Paths) -> [Check; 2] {
    let binary = match std::env::current_exe() {
        Ok(path) => Check::new(Level::Ok, "binary", path.display().to_string()),
        Err(err) => Check::new(Level::Error, "binary", err.to_string()),
    };
    let source = Check::new(
        Level::Info,
        "install source",
        install_source_label(detect_install_source()),
    );
    [binary, source]
}

fn inspect_auth(paths: &Paths) -> [Check; 1] {
    [match auth_state(paths) {
        AuthState::Missing => Check::new(Level::Warn, "auth file", "missing (run `codex login`)"),
        AuthState::Valid => Check::new(Level::Ok, "auth file", "valid"),
        AuthState::Incomplete(reason) => Check::new(Level::Warn, "auth file", reason),
        AuthState::Invalid(reason) => Check::new(
            Level::Error,
            "auth file",
            format!("{reason} (run `codex login`)"),
        ),
    }]
}

fn inspect_profiles_dir(paths: &Paths) -> [Check; 1] {
    [match fs::metadata(&paths.profiles) {
        Ok(meta) if meta.is_dir() => Check::new(
            Level::Ok,
            "profiles directory",
            paths.profiles.display().to_string(),
        ),
        Ok(_) => Check::new(
            Level::Error,
            "profiles directory",
            "exists but is not a directory",
        ),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            Check::new(Level::Info, "profiles directory", "missing")
        }
        Err(err) => Check::new(Level::Error, "profiles directory", err.to_string()),
    }]
}

fn inspect_profiles_index(paths: &Paths) -> [Check; 1] {
    [if !paths.profiles_index.exists() {
        Check::new(Level::Info, "profiles index", "missing")
    } else {
        match profiles_index_len_read_only(&paths.profiles_index) {
            Ok(count) => Check::new(Level::Ok, "profiles index", format!("{} entries", count)),
            Err(err) => Check::new(
                Level::Error,
                "profiles index",
                format!("{err} (run `doctor --fix` or remove profiles.json)"),
            ),
        }
    }]
}

fn inspect_profiles_lock(paths: &Paths) -> [Check; 1] {
    [if !paths.profiles.exists() {
        Check::new(Level::Info, "profiles lock", "not created yet")
    } else if !paths.profiles_lock.exists() {
        Check::new(Level::Info, "profiles lock", "missing")
    } else if !paths.profiles_lock.is_file() {
        Check::new(Level::Error, "profiles lock", "exists but is not a file")
    } else {
        match lock_usage(paths) {
            Ok(_lock) => Check::new(Level::Ok, "profiles lock", "acquired"),
            Err(err) => Check::new(Level::Error, "profiles lock", err),
        }
    }]
}

fn inspect_saved_profiles(paths: &Paths) -> SavedProfilesReport {
    let mut tokens = BTreeMap::new();
    let mut valid = 0usize;
    let mut invalid_ids = Vec::new();

    match profile_paths(&paths.profiles) {
        Ok(paths_list) => {
            for path in paths_list {
                let id = profile_id_from_path(&path).unwrap_or_else(|| path.display().to_string());
                match read_tokens(&path) {
                    Ok(profile_tokens) if is_profile_ready(&profile_tokens) => {
                        valid += 1;
                        tokens.insert(id, Ok(profile_tokens));
                    }
                    Ok(_) => {
                        invalid_ids.push(id.clone());
                        tokens.insert(id, Err("profile is incomplete".to_string()));
                    }
                    Err(err) => {
                        invalid_ids.push(id.clone());
                        tokens.insert(id, Err(err));
                    }
                }
            }
        }
        Err(err) => {
            return SavedProfilesReport {
                check: Check::new(Level::Error, "saved profiles", err),
                tokens,
            };
        }
    }

    let check = if invalid_ids.is_empty() {
        Check::new(
            Level::Ok,
            "saved profiles",
            format!("{} valid, 0 invalid", valid),
        )
    } else {
        Check::new(
            Level::Warn,
            "saved profiles",
            format!(
                "{} valid, {} invalid (remove or re-save invalid profiles)",
                valid,
                invalid_ids.len()
            ),
        )
    };
    SavedProfilesReport { check, tokens }
}

fn inspect_current_profile(
    paths: &Paths,
    tokens: &BTreeMap<String, Result<crate::Tokens, String>>,
) -> Check {
    match auth_state(paths) {
        AuthState::Missing => Check::new(Level::Info, "current profile", "no auth file"),
        AuthState::Incomplete(reason) => Check::new(
            Level::Warn,
            "current profile",
            format!("unavailable ({reason})"),
        ),
        AuthState::Invalid(reason) => Check::new(
            Level::Warn,
            "current profile",
            format!("unavailable ({reason})"),
        ),
        AuthState::Valid => match current_saved_id(paths, tokens) {
            Some(_) => Check::new(Level::Ok, "current profile", "saved"),
            None => Check::new(
                Level::Warn,
                "current profile",
                "not saved (run `codex-profiles save`)",
            ),
        },
    }
}

fn auth_state(paths: &Paths) -> AuthState {
    match read_auth_file(&paths.auth) {
        Ok(_) => match read_tokens(&paths.auth) {
            Ok(tokens) => {
                if is_profile_ready(&tokens) {
                    AuthState::Valid
                } else {
                    AuthState::Incomplete("present but incomplete (run `codex login`)".to_string())
                }
            }
            Err(err) => AuthState::Invalid(err),
        },
        Err(err) => {
            if !paths.auth.exists() {
                AuthState::Missing
            } else {
                AuthState::Invalid(err)
            }
        }
    }
}

fn install_source_label(source: InstallSource) -> &'static str {
    match source {
        InstallSource::Npm => "npm",
        InstallSource::Bun => "bun",
        InstallSource::Brew => "brew",
        InstallSource::Unknown => "unknown",
    }
}

fn profiles_index_len_read_only(path: &Path) -> Result<usize, String> {
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    let count = value
        .get("profiles")
        .and_then(|profiles| {
            profiles
                .as_object()
                .map(|entries| entries.len())
                .or_else(|| profiles.as_array().map(|entries| entries.len()))
        })
        .unwrap_or(0);
    Ok(count)
}

fn profile_paths(profiles_dir: &Path) -> Result<Vec<PathBuf>, String> {
    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }
    if !profiles_dir.is_dir() {
        return Err("profiles directory exists but is not a directory".to_string());
    }

    let mut paths = Vec::new();
    let entries = fs::read_dir(profiles_dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let path = entry.map_err(|err| err.to_string())?.path();
        if !path.is_file() {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("profiles.json") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("update.json") {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        paths.push(path);
    }
    paths.sort();
    Ok(paths)
}
