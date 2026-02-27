use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use fslock::LockFile;
use serde::Deserialize;
use std::fs;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::{
    AUTH_RELOGIN_AND_SAVE, Paths, UI_ERROR_TWO_LINE, UI_INFO_PREFIX, USAGE_ERR_ACCESS_DENIED_403,
    USAGE_ERR_INVALID_RESPONSE, USAGE_ERR_LOCK_ACQUIRE, USAGE_ERR_LOCK_HELD, USAGE_ERR_LOCK_OPEN,
    USAGE_ERR_RATE_LIMITED_429, USAGE_ERR_REQUEST_FAILED_CODE, USAGE_ERR_SERVICE_UNREACHABLE,
    USAGE_ERR_UNAUTHORIZED_401_TITLE, USAGE_UNAVAILABLE_402_DETAIL, USAGE_UNAVAILABLE_402_TITLE,
    USAGE_UNAVAILABLE_DEFAULT, command_name,
};
use crate::{is_plain, style_text, use_color_stdout};

const DEFAULT_BASE_URL: &str = "https://chatgpt.com/backend-api";
const USER_AGENT: &str = "codex-profiles";
const USAGE_RETRY_ATTEMPTS: usize = 3;
const USAGE_RETRY_BASE_MS: u64 = 250;
const USAGE_RETRY_MAX_MS: u64 = 3_000;
const USAGE_RETRY_JITTER_MS: u64 = 125;
#[cfg(not(test))]
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);
const LOCK_RETRY_DELAY: Duration = Duration::from_secs(1);

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
const LOCK_FAIL_ERR: usize = 1;
#[cfg(test)]
const LOCK_FAIL_BUSY: usize = 2;
#[cfg(test)]
static LOCK_FAILPOINT: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Default)]
pub(crate) struct UsageLimits {
    pub(crate) five_hour: Option<UsageWindow>,
    pub(crate) weekly: Option<UsageWindow>,
}

#[derive(Clone, Debug)]
pub(crate) struct UsageWindow {
    pub(crate) left_percent: f64,
    pub(crate) reset_at: i64,
}

#[derive(Debug)]
pub enum UsageFetchError {
    Status(u16),
    Transport(String),
    Parse(String),
}

impl UsageFetchError {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            UsageFetchError::Status(code) => Some(*code),
            _ => None,
        }
    }

    pub fn message(&self) -> String {
        match self {
            UsageFetchError::Status(401) => crate::msg2(
                UI_ERROR_TWO_LINE,
                USAGE_ERR_UNAUTHORIZED_401_TITLE,
                AUTH_RELOGIN_AND_SAVE,
            ),
            UsageFetchError::Status(402) => crate::msg2(
                UI_ERROR_TWO_LINE,
                USAGE_UNAVAILABLE_402_TITLE,
                USAGE_UNAVAILABLE_402_DETAIL,
            ),
            UsageFetchError::Status(403) => USAGE_ERR_ACCESS_DENIED_403.to_string(),
            UsageFetchError::Status(429) => USAGE_ERR_RATE_LIMITED_429.to_string(),
            UsageFetchError::Status(code) => crate::msg1(USAGE_ERR_REQUEST_FAILED_CODE, code),
            UsageFetchError::Transport(err) => crate::msg1(USAGE_ERR_SERVICE_UNREACHABLE, err),
            UsageFetchError::Parse(err) => crate::msg1(USAGE_ERR_INVALID_RESPONSE, err),
        }
    }
}

impl std::fmt::Display for UsageFetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message())
    }
}

#[derive(Debug, Deserialize)]
struct UsagePayload {
    #[serde(default)]
    rate_limit: Option<RateLimitDetails>,
    #[serde(default)]
    additional_rate_limits: Option<Vec<AdditionalRateLimitDetails>>,
}

#[derive(Clone, Debug, Deserialize)]
struct RateLimitDetails {
    #[serde(default)]
    primary_window: Option<RateLimitWindowSnapshot>,
    #[serde(default)]
    secondary_window: Option<RateLimitWindowSnapshot>,
}

#[derive(Clone, Debug, Deserialize)]
struct AdditionalRateLimitDetails {
    #[serde(default)]
    limit_name: Option<String>,
    #[serde(default)]
    metered_feature: Option<String>,
    #[serde(default)]
    rate_limit: Option<RateLimitDetails>,
}

#[derive(Clone, Debug, Deserialize)]
struct RateLimitWindowSnapshot {
    used_percent: f64,
    limit_window_seconds: i64,
    reset_at: i64,
}

#[derive(Clone, Debug)]
struct UsageBucket {
    limit_id: String,
    label: String,
    rate_limit: Option<RateLimitDetails>,
}

pub fn read_base_url(paths: &Paths) -> String {
    let config_path = paths.codex.join("config.toml");
    if let Ok(contents) = fs::read_to_string(config_path) {
        for line in contents.lines() {
            if let Some(value) = parse_config_value(line, "chatgpt_base_url") {
                return normalize_base_url(&value);
            }
        }
    }
    DEFAULT_BASE_URL.to_string()
}

#[doc(hidden)]
pub fn parse_config_value(line: &str, key: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (config_key, raw_value) = line.split_once('=')?;
    if config_key.trim() != key {
        return None;
    }
    let value = strip_inline_comment(raw_value).trim();
    if value.is_empty() {
        return None;
    }
    let value = value.trim_matches('"').trim_matches('\'').trim();
    if value.is_empty() {
        return None;
    }
    Some(value.to_string())
}

fn strip_inline_comment(value: &str) -> &str {
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    for (idx, ch) in value.char_indices() {
        match ch {
            '"' if !in_single && !escape => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            '#' if !in_single && !in_double => return value[..idx].trim_end(),
            _ => {}
        }
        escape = in_double && ch == '\\' && !escape;
        if ch != '\\' {
            escape = false;
        }
    }
    value.trim_end()
}

fn normalize_base_url(value: &str) -> String {
    let mut base = value.trim_end_matches('/').to_string();
    if (base.starts_with("https://chatgpt.com") || base.starts_with("https://chat.openai.com"))
        && !base.contains("/backend-api")
    {
        base = format!("{base}/backend-api");
    }
    base
}

fn usage_endpoint(base_url: &str) -> String {
    if base_url.contains("/backend-api") {
        format!("{base_url}/wham/usage")
    } else {
        format!("{base_url}/api/codex/usage")
    }
}

fn fetch_usage_payload(
    base_url: &str,
    access_token: &str,
    account_id: &str,
) -> Result<UsagePayload, UsageFetchError> {
    let endpoint = usage_endpoint(base_url);
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .http_status_as_error(false)
        .build();
    let agent: ureq::Agent = config.into();
    for attempt in 0..USAGE_RETRY_ATTEMPTS {
        let response = agent
            .get(&endpoint)
            .header("Authorization", &format!("Bearer {access_token}"))
            .header("ChatGPT-Account-Id", account_id)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|err| UsageFetchError::Transport(err.to_string()))?;
        let status = response.status();
        if status == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|value| value.to_str().ok());
            if let Some(delay) = usage_retry_delay(attempt, retry_after) {
                thread::sleep(delay);
                continue;
            }
        }
        if !status.is_success() {
            return Err(UsageFetchError::Status(status.as_u16()));
        }
        return response
            .into_body()
            .read_json::<UsagePayload>()
            .map_err(|err| UsageFetchError::Parse(err.to_string()));
    }
    Err(UsageFetchError::Status(429))
}

fn usage_retry_delay(attempt: usize, retry_after: Option<&str>) -> Option<Duration> {
    if attempt + 1 >= USAGE_RETRY_ATTEMPTS {
        return None;
    }
    if let Some(delay) = retry_after.and_then(parse_retry_after) {
        return Some(delay.min(Duration::from_millis(USAGE_RETRY_MAX_MS)));
    }
    let shift = attempt.min(10) as u32;
    let base = USAGE_RETRY_BASE_MS.saturating_mul(1u64 << shift);
    let mut delay = Duration::from_millis(base.min(USAGE_RETRY_MAX_MS));
    let jitter = usage_retry_jitter();
    delay += jitter;
    Some(delay.min(Duration::from_millis(USAGE_RETRY_MAX_MS)))
}

fn usage_retry_jitter() -> Duration {
    if USAGE_RETRY_JITTER_MS == 0 {
        return Duration::from_millis(0);
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;
    Duration::from_millis(nanos % (USAGE_RETRY_JITTER_MS + 1))
}

fn parse_retry_after(value: &str) -> Option<Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let parsed = chrono::DateTime::parse_from_rfc2822(value).ok()?;
    let retry_at = parsed.with_timezone(&Utc).timestamp();
    let now = Utc::now().timestamp();
    if retry_at <= now {
        return Some(Duration::from_millis(0));
    }
    Some(Duration::from_secs((retry_at - now) as u64))
}

pub fn fetch_usage_details(
    base_url: &str,
    access_token: &str,
    account_id: &str,
    unavailable_text: &str,
    now: DateTime<Local>,
) -> Result<Vec<String>, UsageFetchError> {
    let payload = fetch_usage_payload(base_url, access_token, account_id)?;
    Ok(usage_lines_from_payload(&payload, unavailable_text, now))
}

#[cfg(test)]
fn build_usage_limits(payload: &UsagePayload) -> UsageLimits {
    let buckets = ordered_usage_buckets(usage_buckets(payload));
    let Some(preferred_bucket) = buckets.first() else {
        return UsageLimits::default();
    };
    build_usage_limits_for_rate_limit(preferred_bucket.rate_limit.as_ref())
}

fn usage_lines_from_payload(
    payload: &UsagePayload,
    unavailable_text: &str,
    now: DateTime<Local>,
) -> Vec<String> {
    let buckets = ordered_usage_buckets(usage_buckets(payload));
    if buckets.is_empty() {
        return vec![format_usage_unavailable(
            unavailable_text,
            use_color_stdout(),
        )];
    }
    let multi_bucket = buckets.len() > 1;
    let mut lines = Vec::new();
    for bucket in buckets {
        let limits = build_usage_limits_for_rate_limit(bucket.rate_limit.as_ref());
        let has_data = limits.five_hour.is_some() || limits.weekly.is_some();
        if !has_data {
            continue;
        }
        let mut bucket_lines = format_usage(
            format_limit(limits.five_hour.as_ref(), now, unavailable_text),
            format_limit(limits.weekly.as_ref(), now, unavailable_text),
            unavailable_text,
        );
        if limits.five_hour.is_some() && limits.weekly.is_some() {
            bucket_lines = label_dual_window_lines(bucket_lines);
        }
        if multi_bucket {
            let label = usage_bucket_label(&bucket);
            lines.push(label.to_string());
            lines.extend(bucket_lines.into_iter().map(|line| format!("  {line}")));
        } else {
            lines.extend(bucket_lines);
        }
    }
    if lines.is_empty() {
        vec![format_usage_unavailable(
            unavailable_text,
            use_color_stdout(),
        )]
    } else {
        lines
    }
}

fn label_dual_window_lines(mut lines: Vec<String>) -> Vec<String> {
    if let Some(first) = lines.get_mut(0) {
        *first = format!("5 hour: {first}");
    }
    if let Some(second) = lines.get_mut(1) {
        *second = format!("Weekly: {second}");
    }
    lines
}

fn usage_buckets(payload: &UsagePayload) -> Vec<UsageBucket> {
    let mut buckets = Vec::new();
    if let Some(rate_limit) = payload.rate_limit.clone() {
        buckets.push(UsageBucket {
            limit_id: "codex".to_string(),
            label: "codex".to_string(),
            rate_limit: Some(rate_limit),
        });
    }
    if let Some(additional) = payload.additional_rate_limits.as_ref() {
        buckets.extend(additional.iter().map(|details| {
            let limit_id = details
                .metered_feature
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or("unknown")
                .to_string();
            let label = details
                .limit_name
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .unwrap_or(limit_id.as_str())
                .to_string();
            UsageBucket {
                limit_id,
                label,
                rate_limit: details.rate_limit.clone(),
            }
        }));
    }
    buckets
}

fn ordered_usage_buckets(mut buckets: Vec<UsageBucket>) -> Vec<UsageBucket> {
    if let Some(index) = buckets.iter().position(|bucket| bucket.limit_id == "codex") {
        if index != 0 {
            let preferred = buckets.remove(index);
            buckets.insert(0, preferred);
        }
    }
    buckets
}

fn usage_bucket_label(bucket: &UsageBucket) -> &str {
    if bucket.label.trim().is_empty() {
        "unknown"
    } else {
        bucket.label.as_str()
    }
}

fn build_usage_limits_for_rate_limit(rate_limit: Option<&RateLimitDetails>) -> UsageLimits {
    let mut limits = UsageLimits::default();
    let Some(rate_limit) = rate_limit else {
        return limits;
    };
    let mut windows: Vec<(i64, UsageWindow)> = [
        rate_limit.primary_window.as_ref(),
        rate_limit.secondary_window.as_ref(),
    ]
    .into_iter()
    .flatten()
    .map(|window| (window.limit_window_seconds, usage_window_output(window)))
    .collect();
    if windows.is_empty() {
        return limits;
    }
    windows.sort_by_key(|(secs, _)| *secs);
    if let Some((_, first)) = windows.first() {
        limits.five_hour = Some(first.clone());
    }
    if let Some((_, second)) = windows.get(1) {
        limits.weekly = Some(second.clone());
    }
    limits
}

fn usage_window_output(window: &RateLimitWindowSnapshot) -> UsageWindow {
    let left_percent = (100.0 - window.used_percent).clamp(0.0, 100.0);
    let reset_at = window.reset_at;
    UsageWindow {
        left_percent,
        reset_at,
    }
}

pub(crate) struct UsageLine {
    pub(crate) bar: String,
    pub(crate) percent: String,
    pub(crate) reset: String,
    pub(crate) left_percent: Option<i64>,
}

impl UsageLine {
    fn unavailable(text: &str) -> Self {
        UsageLine {
            bar: text.to_string(),
            percent: String::new(),
            reset: String::new(),
            left_percent: None,
        }
    }
}

pub(crate) fn format_limit(
    window: Option<&UsageWindow>,
    now: DateTime<Local>,
    unavailable_text: &str,
) -> UsageLine {
    let Some(window) = window else {
        return UsageLine::unavailable(unavailable_text);
    };
    let left_percent = window.left_percent;
    let left_percent_rounded = left_percent.round() as i64;
    let bar = render_bar(left_percent);
    let bar = style_usage_bar(&bar, left_percent);
    let percent = format!("{left_percent_rounded}%");
    let reset =
        format_reset_timestamp(window.reset_at, now).unwrap_or_else(|| "unknown".to_string());
    UsageLine {
        bar,
        percent,
        reset,
        left_percent: Some(left_percent_rounded),
    }
}

pub fn usage_unavailable() -> &'static str {
    USAGE_UNAVAILABLE_DEFAULT
}

pub fn format_usage_unavailable(text: &str, use_color: bool) -> String {
    if is_plain() {
        crate::msg1(UI_INFO_PREFIX, text)
    } else if use_color {
        text.red().bold().to_string()
    } else {
        text.to_string()
    }
}

pub(crate) fn format_usage(
    five: UsageLine,
    weekly: UsageLine,
    unavailable_text: &str,
) -> Vec<String> {
    let use_color = use_color_stdout();
    let available: Vec<UsageLine> = [five, weekly]
        .into_iter()
        .filter(|line| line.left_percent.is_some())
        .collect();
    if available.is_empty() {
        return vec![format_usage_unavailable(unavailable_text, use_color)];
    }
    let has_zero = available.iter().any(|line| line.left_percent == Some(0));
    let multiple = available.len() > 1;
    available
        .into_iter()
        .map(|line| {
            let dim = use_color && multiple && has_zero && line.left_percent != Some(0);
            format_usage_line(&line, dim, use_color)
        })
        .collect()
}

pub(crate) fn format_reset_timestamp(reset_at: i64, now: DateTime<Local>) -> Option<String> {
    let reset_at = local_from_timestamp(reset_at)?;
    let time = reset_at.format("%H:%M").to_string();
    if reset_at.date_naive() == now.date_naive() {
        Some(time)
    } else {
        Some(format!("{time} on {}", reset_at.format("%-d %b")))
    }
}

fn format_usage_line(line: &UsageLine, dim: bool, use_color: bool) -> String {
    let reset = reset_label(&line.reset);
    let reset = reset.to_string();
    let percent = if line.percent.is_empty() {
        String::new()
    } else {
        format!("{} left", line.percent)
    };
    let resets = format_resets_suffix(&reset, use_color);
    if is_plain() {
        let mut out = String::new();
        if !percent.is_empty() {
            out.push_str(&percent);
        }
        if !resets.is_empty() {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&resets);
        }
        return out;
    }
    let resets = if resets.is_empty() {
        resets
    } else {
        format!(" {resets}")
    };
    let bar = if dim {
        strip_ansi(&line.bar)
    } else {
        line.bar.clone()
    };
    let formatted = if percent.is_empty() {
        format!("{bar}{resets}")
    } else {
        format!("{bar} {percent}{resets}")
    };
    if dim && use_color {
        formatted.dimmed().to_string()
    } else {
        formatted
    }
}

fn reset_label(reset: &str) -> &str {
    if reset.is_empty() { "unknown" } else { reset }
}

fn format_resets_suffix(reset: &str, use_color: bool) -> String {
    let text = format!("(resets {reset})");
    style_text(&text, use_color, |text| text.dimmed().italic())
}

fn render_bar(left_percent: f64) -> String {
    let total = 20;
    let filled = ((left_percent / 100.0) * total as f64).round() as usize;
    let filled = filled.min(total);
    let empty = total.saturating_sub(filled);
    format!(
        "{}{}",
        "▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮"
            .chars()
            .take(filled)
            .collect::<String>(),
        "▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯"
            .chars()
            .take(empty)
            .collect::<String>()
    )
}

fn style_usage_bar(bar: &str, left_percent: f64) -> String {
    if !use_color_stdout() {
        return bar.to_string();
    }
    if left_percent >= 66.0 {
        bar.green().to_string()
    } else if left_percent >= 33.0 {
        bar.yellow().to_string()
    } else {
        bar.red().to_string()
    }
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    loop {
        let Some(ch) = chars.next() else {
            break;
        };
        if ch == '\x1b' && consume_ansi_escape(&mut chars) {
            continue;
        }
        out.push(ch);
    }
    out
}

fn consume_ansi_escape<I>(chars: &mut std::iter::Peekable<I>) -> bool
where
    I: Iterator<Item = char>,
{
    if chars.peek() != Some(&'[') {
        return false;
    }
    chars.next();
    for c in chars.by_ref() {
        if c == 'm' {
            break;
        }
    }
    true
}

fn local_from_timestamp(ts: i64) -> Option<DateTime<Local>> {
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0)?;
    Some(dt.with_timezone(&Local))
}

#[derive(Debug)]
pub struct UsageLock {
    _lock: LockFile,
}

pub fn lock_usage(paths: &Paths) -> Result<UsageLock, String> {
    let start = Instant::now();
    let mut lock = LockFile::open(&paths.profiles_lock)
        .map_err(|err| crate::msg1(USAGE_ERR_LOCK_OPEN, err))?;
    loop {
        match try_lock(&mut lock) {
            Ok(true) => break,
            Ok(false) => {
                if start.elapsed() > lock_timeout() {
                    return Err(crate::msg1(USAGE_ERR_LOCK_ACQUIRE, command_name()));
                }
                thread::sleep(LOCK_RETRY_DELAY);
            }
            Err(err) => {
                return Err(crate::msg1(USAGE_ERR_LOCK_HELD, err));
            }
        }
    }
    Ok(UsageLock { _lock: lock })
}

#[cfg(not(test))]
fn lock_timeout() -> Duration {
    LOCK_TIMEOUT
}

#[cfg(not(test))]
fn try_lock(lock: &mut LockFile) -> Result<bool, fslock::Error> {
    lock.try_lock()
}

#[cfg(test)]
fn lock_timeout() -> Duration {
    Duration::from_millis(50)
}

#[cfg(test)]
fn try_lock(lock: &mut LockFile) -> Result<bool, fslock::Error> {
    match LOCK_FAILPOINT.load(Ordering::Relaxed) {
        LOCK_FAIL_ERR => Err(std::io::Error::other("fail")),
        LOCK_FAIL_BUSY => Ok(false),
        _ => lock.try_lock(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{
        http_ok_response, make_paths, set_env_guard, set_plain_guard, spawn_server,
    };
    use std::fs;
    use std::sync::Mutex;

    static LOCK_TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn config_parsing_paths() {
        assert!(parse_config_value("", "key").is_none());
        assert!(parse_config_value("# comment", "key").is_none());
        assert!(parse_config_value("other = 1", "key").is_none());
        assert!(parse_config_value("key =", "key").is_none());
        assert_eq!(
            parse_config_value("key = 'value'", "key"),
            Some("value".to_string())
        );
        assert_eq!(strip_inline_comment("value # comment"), "value");
    }

    #[test]
    fn normalize_base_url_and_endpoint() {
        let url = normalize_base_url("https://chatgpt.com");
        assert!(url.ends_with("/backend-api"));
        assert!(usage_endpoint(&url).contains("wham/usage"));
        assert!(usage_endpoint("http://example.com").contains("api/codex/usage"));
    }

    #[test]
    fn fetch_usage_payload_paths() {
        let payload = r#"{"rate_limit":{"primary_window":{"used_percent":50.0,"limit_window_seconds":3600,"reset_at":1}}}"#;
        let resp = http_ok_response(payload, "application/json");
        let url = spawn_server(resp);
        let base_url = format!("{url}/backend-api");
        fetch_usage_payload(&base_url, "token", "acct").unwrap();

        let err_resp =
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n".to_string();
        let err_url = spawn_server(err_resp);
        let base_url = format!("{err_url}/backend-api");
        let err = fetch_usage_payload(&base_url, "token", "acct").unwrap_err();
        assert!(matches!(err, UsageFetchError::Status(_)));

        let bad_resp =
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 1\r\n\r\n{"
                .to_string();
        let bad_url = spawn_server(bad_resp);
        let base_url = format!("{bad_url}/backend-api");
        let err = fetch_usage_payload(&base_url, "token", "acct").unwrap_err();
        assert!(matches!(err, UsageFetchError::Parse(_)));
    }

    #[test]
    fn fetch_usage_details_paths() {
        let payload = r#"{"rate_limit":{"primary_window":{"used_percent":10.0,"limit_window_seconds":3600,"reset_at":1}}}"#;
        let resp = http_ok_response(payload, "application/json");
        let url = spawn_server(resp);
        let base_url = format!("{url}/backend-api");
        let lines =
            fetch_usage_details(&base_url, "token", "acct", "unavailable", Local::now()).unwrap();
        assert!(!lines.is_empty());
    }

    #[test]
    fn retry_after_parsing_paths() {
        assert_eq!(parse_retry_after("2"), Some(Duration::from_secs(2)));
        assert!(parse_retry_after("Thu, 01 Jan 1970 00:00:00 GMT").is_some());
        assert!(parse_retry_after("not-a-date").is_none());
        assert!(usage_retry_delay(USAGE_RETRY_ATTEMPTS - 1, Some("1")).is_none());
        assert!(usage_retry_delay(0, Some("2")).is_some());
    }

    #[test]
    fn usage_limits_and_formatting() {
        let payload = UsagePayload {
            rate_limit: None,
            additional_rate_limits: None,
        };
        let limits = build_usage_limits(&payload);
        assert!(limits.five_hour.is_none());

        let window = RateLimitWindowSnapshot {
            used_percent: 50.0,
            limit_window_seconds: 10,
            reset_at: Local::now().timestamp(),
        };
        let rate_limit = RateLimitDetails {
            primary_window: Some(window.clone()),
            secondary_window: Some(window.clone()),
        };
        let payload = UsagePayload {
            rate_limit: Some(rate_limit),
            additional_rate_limits: None,
        };
        let limits = build_usage_limits(&payload);
        assert!(limits.five_hour.is_some());
        let line = format_limit(limits.five_hour.as_ref(), Local::now(), "none");
        assert!(line.left_percent.is_some());
    }

    #[test]
    fn usage_limits_fallback_to_additional_bucket_when_primary_missing() {
        let window = RateLimitWindowSnapshot {
            used_percent: 25.0,
            limit_window_seconds: 900,
            reset_at: Local::now().timestamp(),
        };
        let payload = UsagePayload {
            rate_limit: None,
            additional_rate_limits: Some(vec![AdditionalRateLimitDetails {
                limit_name: Some("codex_other".to_string()),
                metered_feature: Some("codex_other".to_string()),
                rate_limit: Some(RateLimitDetails {
                    primary_window: Some(window),
                    secondary_window: None,
                }),
            }]),
        };
        let limits = build_usage_limits(&payload);
        assert!(limits.five_hour.is_some());
    }

    #[test]
    fn usage_lines_include_multi_bucket_labels() {
        let _plain = set_plain_guard(true);
        let now = Local::now();
        let payload = UsagePayload {
            rate_limit: Some(RateLimitDetails {
                primary_window: Some(RateLimitWindowSnapshot {
                    used_percent: 20.0,
                    limit_window_seconds: 18000,
                    reset_at: now.timestamp() + 600,
                }),
                secondary_window: None,
            }),
            additional_rate_limits: Some(vec![AdditionalRateLimitDetails {
                limit_name: Some("codex_other".to_string()),
                metered_feature: Some("codex_other".to_string()),
                rate_limit: Some(RateLimitDetails {
                    primary_window: Some(RateLimitWindowSnapshot {
                        used_percent: 60.0,
                        limit_window_seconds: 3600,
                        reset_at: now.timestamp() + 900,
                    }),
                    secondary_window: None,
                }),
            }]),
        };
        let lines = usage_lines_from_payload(&payload, "unavailable", now);
        assert!(lines.iter().any(|line| line == "codex"));
        assert!(lines.iter().any(|line| line == "codex_other"));
        assert!(
            lines
                .iter()
                .any(|line| line.starts_with("  ") && line.contains("left"))
        );
    }

    #[test]
    fn usage_lines_label_dual_windows_for_single_bucket() {
        let _plain = set_plain_guard(true);
        let now = Local::now();
        let payload = UsagePayload {
            rate_limit: Some(RateLimitDetails {
                primary_window: Some(RateLimitWindowSnapshot {
                    used_percent: 20.0,
                    limit_window_seconds: 18000,
                    reset_at: now.timestamp() + 600,
                }),
                secondary_window: Some(RateLimitWindowSnapshot {
                    used_percent: 50.0,
                    limit_window_seconds: 604800,
                    reset_at: now.timestamp() + 3600,
                }),
            }),
            additional_rate_limits: None,
        };
        let lines = usage_lines_from_payload(&payload, "unavailable", now);
        assert!(lines.iter().any(|line| line.starts_with("5 hour: ")));
        assert!(lines.iter().any(|line| line.starts_with("Weekly: ")));
    }

    #[test]
    fn usage_unavailable_paths() {
        let _plain = set_plain_guard(true);
        assert_eq!(usage_unavailable(), "Data not available");
        let text = format_usage_unavailable("text", false);
        assert!(text.contains("Info"));
    }

    #[test]
    fn format_usage_variants() {
        let unavailable = "unavailable";
        let lines = format_usage(
            UsageLine::unavailable(unavailable),
            UsageLine::unavailable(unavailable),
            unavailable,
        );
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn format_usage_line_plain_and_dim() {
        let line = UsageLine {
            bar: render_bar(50.0),
            percent: "50%".to_string(),
            reset: "soon".to_string(),
            left_percent: Some(50),
        };
        let _plain = set_plain_guard(true);
        let plain = format_usage_line(&line, false, false);
        assert!(plain.contains("left"));
    }

    #[test]
    fn style_bar_and_strip_ansi() {
        let _env = set_env_guard("NO_COLOR", Some("1"));
        let bar = render_bar(10.0);
        let styled = style_usage_bar(&bar, 10.0);
        assert_eq!(bar, styled);
        let stripped = strip_ansi("\x1b[31mred\x1b[0m");
        assert_eq!(stripped, "red");
    }

    #[test]
    fn format_reset_timestamp_helpers() {
        use chrono::Timelike;
        let now = Local::now()
            .with_hour(12)
            .and_then(|value| value.with_minute(0))
            .and_then(|value| value.with_second(0))
            .and_then(|value| value.with_nanosecond(0))
            .expect("valid midday");
        let same_day = format_reset_timestamp(now.timestamp() + 60, now).expect("same day");
        let cross_day =
            format_reset_timestamp(now.timestamp() + 60 * 60 * 24, now).expect("cross day");
        assert!(same_day.contains(':'));
        assert!(!same_day.contains(" on "));
        assert!(cross_day.contains(" on "));
        assert!(local_from_timestamp(0).is_some());
        assert!(local_from_timestamp(-1).is_some());
    }

    #[test]
    fn lock_usage_failure_paths() {
        let _guard = LOCK_TEST_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = make_paths(dir.path());
        fs::create_dir_all(&paths.profiles).unwrap();
        fs::write(&paths.profiles_lock, "").unwrap();

        LOCK_FAILPOINT.store(LOCK_FAIL_BUSY, Ordering::Relaxed);
        let err = lock_usage(&paths).unwrap_err();
        assert!(err.contains("Could not acquire profiles lock"));
        LOCK_FAILPOINT.store(LOCK_FAIL_ERR, Ordering::Relaxed);
        let err = lock_usage(&paths).unwrap_err();
        assert!(err.contains("Could not lock profiles file"));
        LOCK_FAILPOINT.store(0, Ordering::Relaxed);
    }

    #[test]
    fn lock_usage_open_error() {
        let _guard = LOCK_TEST_MUTEX.lock().unwrap();
        let dir = tempfile::tempdir().expect("tempdir");
        let lock_dir = dir.path().join("locked");
        fs::create_dir_all(&lock_dir).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&lock_dir, fs::Permissions::from_mode(0o400)).unwrap();
        }
        let mut paths = make_paths(dir.path());
        paths.profiles_lock = lock_dir.join("profiles.lock");
        let err = lock_usage(&paths).unwrap_err();
        assert!(err.contains("Could not open profiles lock"));
    }
}
