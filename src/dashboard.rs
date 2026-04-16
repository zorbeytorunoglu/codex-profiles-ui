use std::io::{self, IsTerminal as _};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Local};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap};

use crate::{
    DashboardProfile, DashboardSnapshot, Paths, active_profile_unsaved_reason,
    collect_dashboard_snapshot, command_name, is_plain, load_saved_profile_by_id,
    save_current_profile_internal,
};

const POLL_INTERVAL: Duration = Duration::from_millis(200);

pub fn run_dashboard(paths: &Paths, interval_secs: u64, json: bool) -> Result<(), String> {
    if json {
        return Err("Error: `dashboard` is interactive and does not support `--json`.".to_string());
    }
    require_dashboard_tty()?;

    enable_raw_mode().map_err(|err| format!("Error: Could not enable raw mode: {err}"))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)
        .map_err(|err| format!("Error: Could not initialize dashboard: {err}"))?;

    let _guard = TerminalGuard;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|err| format!("Error: Could not create dashboard: {err}"))?;

    let mut app = DashboardApp::new(paths.clone(), Duration::from_secs(interval_secs));
    app.run(&mut terminal)
}

fn require_dashboard_tty() -> Result<(), String> {
    if io::stdin().is_terminal() && io::stdout().is_terminal() {
        return Ok(());
    }
    Err(format!(
        "Error: Dashboard requires an interactive TTY. Run `{} dashboard` in a terminal.",
        command_name()
    ))
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, Show);
    }
}

struct WorkerHandle {
    commands: Sender<WorkerCommand>,
    events: Receiver<WorkerEvent>,
}

enum WorkerCommand {
    Refresh,
    Load {
        profile_id: String,
        save_before_load: bool,
    },
}

enum WorkerEvent {
    Refreshed(Result<DashboardSnapshot, String>),
    LoadFinished(Result<LoadOutcome, String>),
}

struct LoadOutcome {
    display_name: String,
    warning: Option<String>,
}

struct DashboardApp {
    paths: Paths,
    worker: WorkerHandle,
    refresh_interval: Duration,
    next_refresh_at: Instant,
    profiles: Vec<DashboardProfile>,
    table_state: TableState,
    last_refresh_at: Option<DateTime<Local>>,
    base_url_error: Option<String>,
    banner: Option<Banner>,
    busy: BusyState,
    confirm_load: Option<PendingLoadConfirmation>,
}

#[derive(Clone)]
struct PendingLoadConfirmation {
    profile_id: String,
    display_name: String,
    reason: String,
}

#[derive(Clone, Copy)]
enum BusyState {
    Idle,
    Refreshing,
    Loading,
}

struct Banner {
    kind: BannerKind,
    text: String,
}

#[derive(Clone, Copy)]
enum BannerKind {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Copy)]
enum ProfileState {
    Ok,
    Warning,
    Unavailable,
    Error,
}

impl DashboardApp {
    fn new(paths: Paths, refresh_interval: Duration) -> Self {
        Self {
            worker: spawn_worker(paths.clone()),
            paths,
            refresh_interval,
            next_refresh_at: Instant::now(),
            profiles: Vec::new(),
            table_state: TableState::default(),
            last_refresh_at: None,
            base_url_error: None,
            banner: Some(Banner {
                kind: BannerKind::Info,
                text: "Loading profile status...".to_string(),
            }),
            busy: BusyState::Idle,
            confirm_load: None,
        }
    }

    fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), String> {
        self.start_refresh()?;

        loop {
            self.process_worker_events()?;
            if self.should_auto_refresh() {
                self.start_refresh()?;
            }

            terminal
                .draw(|frame| self.draw(frame))
                .map_err(|err| format!("Error: Could not draw dashboard: {err}"))?;

            if !event::poll(POLL_INTERVAL)
                .map_err(|err| format!("Error: Could not read dashboard input: {err}"))?
            {
                continue;
            }

            let Event::Key(key) = event::read()
                .map_err(|err| format!("Error: Could not read dashboard input: {err}"))?
            else {
                continue;
            };
            if !matches!(key.kind, KeyEventKind::Press | KeyEventKind::Repeat) {
                continue;
            }

            if self.handle_key(key)? {
                return Ok(());
            }
        }
    }

    fn process_worker_events(&mut self) -> Result<(), String> {
        loop {
            match self.worker.events.try_recv() {
                Ok(WorkerEvent::Refreshed(result)) => {
                    self.busy = BusyState::Idle;
                    match result {
                        Ok(snapshot) => self.apply_snapshot(snapshot),
                        Err(err) => self.set_banner(BannerKind::Error, err),
                    }
                }
                Ok(WorkerEvent::LoadFinished(result)) => {
                    self.busy = BusyState::Idle;
                    match result {
                        Ok(outcome) => {
                            if let Some(warning) = outcome.warning {
                                self.set_banner(BannerKind::Warning, warning);
                            } else {
                                self.set_banner(
                                    BannerKind::Success,
                                    format!("Loaded profile {}", outcome.display_name),
                                );
                            }
                            self.start_refresh()?;
                        }
                        Err(err) => self.set_banner(BannerKind::Error, err),
                    }
                }
                Err(TryRecvError::Empty) => return Ok(()),
                Err(TryRecvError::Disconnected) => {
                    return Err("Error: Dashboard worker stopped unexpectedly.".to_string());
                }
            }
        }
    }

    fn apply_snapshot(&mut self, snapshot: DashboardSnapshot) {
        let selected_key = self.selected_key();
        self.last_refresh_at = Some(snapshot.refreshed_at);
        self.base_url_error = snapshot.base_url_error;
        self.profiles = snapshot.profiles;
        self.restore_selection(selected_key);
        self.next_refresh_at = Instant::now() + self.refresh_interval;
        if matches!(
            self.banner.as_ref().map(|banner| banner.kind),
            Some(BannerKind::Info)
        ) {
            self.banner = None;
        }

        if self.profiles.is_empty() && self.banner.is_none() {
            self.set_banner(
                BannerKind::Info,
                "No saved profiles yet. Save a profile to start monitoring it.".to_string(),
            );
        }
    }

    fn should_auto_refresh(&self) -> bool {
        self.confirm_load.is_none()
            && matches!(self.busy, BusyState::Idle)
            && Instant::now() >= self.next_refresh_at
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        if matches!(
            (key.code, key.modifiers),
            (KeyCode::Char('c'), modifiers) if modifiers.contains(KeyModifiers::CONTROL)
        ) {
            return Ok(true);
        }

        if self.confirm_load.is_some() {
            return self.handle_confirmation_key(key);
        }

        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Up | KeyCode::Char('k') => {
                self.select_previous();
                Ok(false)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.select_next();
                Ok(false)
            }
            KeyCode::Char('r') => {
                if matches!(self.busy, BusyState::Idle) {
                    self.start_refresh()?;
                }
                Ok(false)
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                self.request_load_selected()?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn handle_confirmation_key(&mut self, key: KeyEvent) -> Result<bool, String> {
        let Some(confirm) = self.confirm_load.clone() else {
            return Ok(false);
        };
        match key.code {
            KeyCode::Char('q') => Ok(true),
            KeyCode::Esc => {
                self.confirm_load = None;
                self.set_banner(BannerKind::Info, "Load cancelled.".to_string());
                Ok(false)
            }
            KeyCode::Char('s') | KeyCode::Char('S') => {
                let profile_id = confirm.profile_id.clone();
                self.confirm_load = None;
                self.start_load(profile_id, true)?;
                Ok(false)
            }
            KeyCode::Char('f') | KeyCode::Char('F') => {
                let profile_id = confirm.profile_id.clone();
                self.confirm_load = None;
                self.start_load(profile_id, false)?;
                Ok(false)
            }
            _ => Ok(false),
        }
    }

    fn request_load_selected(&mut self) -> Result<(), String> {
        if !matches!(self.busy, BusyState::Idle) {
            return Ok(());
        }

        let Some(profile) = self.selected_profile() else {
            return Ok(());
        };
        if profile.is_current {
            self.set_banner(
                BannerKind::Info,
                "Selected profile is already active.".to_string(),
            );
            return Ok(());
        }

        let Some(profile_id) = profile.id.clone() else {
            self.set_banner(
                BannerKind::Warning,
                "Only saved profiles can be loaded.".to_string(),
            );
            return Ok(());
        };

        if let Some(reason) = active_profile_unsaved_reason(&self.paths)? {
            self.confirm_load = Some(PendingLoadConfirmation {
                profile_id,
                display_name: profile_title(profile),
                reason,
            });
            return Ok(());
        }

        self.start_load(profile_id, false)
    }

    fn start_refresh(&mut self) -> Result<(), String> {
        self.busy = BusyState::Refreshing;
        self.worker
            .commands
            .send(WorkerCommand::Refresh)
            .map_err(|_| "Error: Dashboard worker stopped unexpectedly.".to_string())
    }

    fn start_load(&mut self, profile_id: String, save_before_load: bool) -> Result<(), String> {
        self.busy = BusyState::Loading;
        self.worker
            .commands
            .send(WorkerCommand::Load {
                profile_id,
                save_before_load,
            })
            .map_err(|_| "Error: Dashboard worker stopped unexpectedly.".to_string())
    }

    fn selected_key(&self) -> Option<String> {
        let profile = self.selected_profile()?;
        Some(profile_key(profile))
    }

    fn restore_selection(&mut self, selected_key: Option<String>) {
        let selected = selected_key
            .and_then(|key| {
                self.profiles
                    .iter()
                    .position(|profile| profile_key(profile) == key)
            })
            .or_else(|| (!self.profiles.is_empty()).then_some(0));
        self.table_state.select(selected);
    }

    fn select_previous(&mut self) {
        if self.profiles.is_empty() {
            self.table_state.select(None);
            return;
        }
        let next = match self.table_state.selected() {
            Some(0) | None => self.profiles.len() - 1,
            Some(index) => index.saturating_sub(1),
        };
        self.table_state.select(Some(next));
    }

    fn select_next(&mut self) {
        if self.profiles.is_empty() {
            self.table_state.select(None);
            return;
        }
        let next = match self.table_state.selected() {
            Some(index) if index + 1 < self.profiles.len() => index + 1,
            _ => 0,
        };
        self.table_state.select(Some(next));
    }

    fn selected_profile(&self) -> Option<&DashboardProfile> {
        self.table_state
            .selected()
            .and_then(|index| self.profiles.get(index))
    }

    fn set_banner(&mut self, kind: BannerKind, text: String) {
        self.banner = Some(Banner { kind, text });
    }

    fn draw(&mut self, frame: &mut Frame<'_>) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(8),
                Constraint::Length(2),
            ])
            .split(frame.area());

        self.draw_header(frame, areas[0]);
        self.draw_body(frame, areas[1]);
        self.draw_footer(frame, areas[2]);

        if self.confirm_load.is_some() {
            self.draw_confirmation(frame);
        }
    }

    fn draw_header(&self, frame: &mut Frame<'_>, area: Rect) {
        let title = Line::from(vec![Span::styled(
            "Codex Profiles Dashboard",
            Style::default().add_modifier(Modifier::BOLD),
        )]);
        let meta = Line::from(self.header_status_text());
        let banner = match self.banner_text() {
            Some((style, text)) => Line::from(Span::styled(text, style)),
            None => Line::from(""),
        };
        let block = Block::default().borders(Borders::ALL).title("Overview");
        let paragraph = Paragraph::new(vec![title, meta, banner]).block(block);
        frame.render_widget(paragraph, area);
    }

    fn draw_body(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(area);

        self.draw_table(frame, areas[0]);
        self.draw_details(frame, areas[1]);
    }

    fn draw_table(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if self.profiles.is_empty() {
            let block = Block::default().borders(Borders::ALL).title("Profiles");
            let paragraph = Paragraph::new("No saved profiles to display.")
                .block(block)
                .wrap(Wrap { trim: false });
            frame.render_widget(paragraph, area);
            return;
        }

        let header = Row::new(vec![
            Cell::from("A"),
            Cell::from("Profile"),
            Cell::from("Account"),
            Cell::from("Plan"),
            Cell::from("State"),
            Cell::from("Usage"),
        ])
        .style(Style::default().add_modifier(Modifier::BOLD));

        let rows = self.profiles.iter().map(|profile| {
            let state = profile_state(profile);
            let usage = usage_overview(profile);
            let state_style = state_style(state);
            Row::new(vec![
                Cell::from(if profile.is_current { "*" } else { "" }),
                Cell::from(profile_title(profile)),
                Cell::from(profile_account(profile)),
                Cell::from(profile_plan(profile)),
                Cell::from(Span::styled(state.label(), state_style)),
                Cell::from(usage),
            ])
        });

        let highlight = if is_plain() {
            Style::default().add_modifier(Modifier::REVERSED)
        } else {
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD)
        };
        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Min(14),
                Constraint::Min(20),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Min(20),
            ],
        )
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("Profiles"))
        .highlight_style(highlight)
        .column_spacing(1);

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn draw_details(&self, frame: &mut Frame<'_>, area: Rect) {
        let title = self
            .selected_profile()
            .map(profile_title)
            .unwrap_or_else(|| "Details".to_string());
        let block = Block::default().borders(Borders::ALL).title(title);
        let text = if let Some(profile) = self.selected_profile() {
            selected_profile_text(profile)
        } else if matches!(self.busy, BusyState::Refreshing) {
            "Refreshing profile status...".to_string()
        } else {
            "Select a profile to inspect it.".to_string()
        };
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn draw_footer(&self, frame: &mut Frame<'_>, area: Rect) {
        let text = if self.confirm_load.is_some() {
            "s save active and load  f force load  esc cancel  q quit"
        } else {
            "up/down move  enter load  r refresh  q quit"
        };
        let paragraph =
            Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Keys"));
        frame.render_widget(paragraph, area);
    }

    fn draw_confirmation(&self, frame: &mut Frame<'_>) {
        let Some(confirm) = self.confirm_load.as_ref() else {
            return;
        };
        let area = centered_rect(70, 32, frame.area());
        frame.render_widget(Clear, area);
        let lines = vec![
            Line::from(vec![Span::styled(
                "Active profile has unsaved changes",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(format!(
                "Loading {} would replace the current auth.json.",
                confirm.display_name
            )),
            Line::from(confirm.reason.clone()),
            Line::from(""),
            Line::from("Press `s` to save the current profile first."),
            Line::from("Press `f` to continue without saving."),
            Line::from("Press `Esc` to cancel."),
        ];
        let paragraph = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Confirm Load"))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }

    fn banner_text(&self) -> Option<(Style, String)> {
        if let Some(banner) = &self.banner {
            return Some((banner_style(banner.kind), banner.text.clone()));
        }
        self.base_url_error
            .as_ref()
            .map(|message| (banner_style(BannerKind::Warning), message.clone()))
    }

    fn header_status_text(&self) -> String {
        let last = self
            .last_refresh_at
            .map(|time| time.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "pending".to_string());
        let next = if matches!(self.busy, BusyState::Idle) {
            format_duration(
                self.next_refresh_at
                    .saturating_duration_since(Instant::now()),
            )
        } else {
            "--:--".to_string()
        };
        format!(
            "Profiles: {} | Last refresh: {} | Next refresh: {} | {}",
            self.profiles.len(),
            last,
            next,
            self.busy.label()
        )
    }
}

impl BusyState {
    fn label(self) -> &'static str {
        match self {
            BusyState::Idle => "Idle",
            BusyState::Refreshing => "Refreshing",
            BusyState::Loading => "Loading profile",
        }
    }
}

impl ProfileState {
    fn label(self) -> &'static str {
        match self {
            ProfileState::Ok => "OK",
            ProfileState::Warning => "Warning",
            ProfileState::Unavailable => "Unavailable",
            ProfileState::Error => "Error",
        }
    }
}

fn spawn_worker(paths: Paths) -> WorkerHandle {
    let (commands_tx, commands_rx) = mpsc::channel();
    let (events_tx, events_rx) = mpsc::channel();

    thread::spawn(move || {
        while let Ok(command) = commands_rx.recv() {
            let event = match command {
                WorkerCommand::Refresh => {
                    WorkerEvent::Refreshed(collect_dashboard_snapshot(&paths))
                }
                WorkerCommand::Load {
                    profile_id,
                    save_before_load,
                } => {
                    let result = (|| {
                        if save_before_load {
                            save_current_profile_internal(&paths, None)?;
                        }
                        let loaded = load_saved_profile_by_id(&paths, &profile_id)?;
                        Ok(LoadOutcome {
                            display_name: loaded
                                .label
                                .clone()
                                .or(loaded.email.clone())
                                .unwrap_or_else(|| loaded.id.clone()),
                            warning: loaded.warning,
                        })
                    })();
                    WorkerEvent::LoadFinished(result)
                }
            };

            if events_tx.send(event).is_err() {
                break;
            }
        }
    });

    WorkerHandle {
        commands: commands_tx,
        events: events_rx,
    }
}

fn profile_key(profile: &DashboardProfile) -> String {
    match profile.id.as_deref() {
        Some(id) => format!("saved:{id}"),
        None if profile.is_current => "current:unsaved".to_string(),
        None => format!("transient:{}", profile.display),
    }
}

fn profile_title(profile: &DashboardProfile) -> String {
    profile
        .label
        .clone()
        .or(profile.email.clone())
        .or(profile.id.clone())
        .unwrap_or_else(|| "Unknown profile".to_string())
}

fn profile_account(profile: &DashboardProfile) -> String {
    profile
        .email
        .clone()
        .or(profile.id.clone())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn profile_plan(profile: &DashboardProfile) -> String {
    if profile.is_api_key {
        return "API key".to_string();
    }
    profile.plan.clone().unwrap_or_else(|| "-".to_string())
}

fn profile_state(profile: &DashboardProfile) -> ProfileState {
    if profile.error_summary.is_some()
        || profile
            .usage
            .as_ref()
            .map(|usage| usage.state == "error")
            .unwrap_or(false)
    {
        return ProfileState::Error;
    }
    if !profile.warnings.is_empty() {
        return ProfileState::Warning;
    }
    match profile.usage.as_ref().map(|usage| usage.state) {
        Some("unavailable") => ProfileState::Unavailable,
        Some("ok") | None => ProfileState::Ok,
        Some(_) => ProfileState::Warning,
    }
}

fn usage_overview(profile: &DashboardProfile) -> String {
    if let Some(summary) = profile.error_summary.as_deref() {
        return truncate_text(summary, 64);
    }
    let Some(usage) = profile.usage.as_ref() else {
        return "-".to_string();
    };
    if usage.state != "ok" {
        return truncate_text(usage.summary.as_deref().unwrap_or("Data not available"), 64);
    }

    let parts: Vec<String> = usage.buckets.iter().filter_map(bucket_overview).collect();
    if parts.is_empty() {
        truncate_text(usage.summary.as_deref().unwrap_or("Data not available"), 64)
    } else {
        truncate_text(&parts.join(" | "), 64)
    }
}

fn bucket_overview(bucket: &crate::usage::UsageSnapshotBucket) -> Option<String> {
    let mut windows = Vec::new();
    if let Some(window) = bucket.five_hour.as_ref() {
        windows.push(format!("5h {}%", window.left_percent));
    }
    if let Some(window) = bucket.weekly.as_ref() {
        windows.push(format!("7d {}%", window.left_percent));
    }
    if windows.is_empty() {
        return None;
    }
    let summary = windows.join(" / ");
    if bucket.label.eq_ignore_ascii_case("default") {
        Some(summary)
    } else {
        Some(format!("{} {summary}", bucket.label))
    }
}

fn selected_profile_text(profile: &DashboardProfile) -> String {
    let mut lines = vec![
        format!("Account: {}", profile_account(profile)),
        format!("Plan: {}", profile_plan(profile)),
        format!("Saved: {}", if profile.is_saved { "yes" } else { "no" }),
        format!("State: {}", profile_state(profile).label()),
    ];
    if let Some(id) = profile.id.as_deref() {
        lines.push(format!("ID: {id}"));
    }
    if let Some(usage) = profile.usage.as_ref() {
        if let Some(status_code) = usage.status_code {
            lines.push(format!("Usage status code: {status_code}"));
        }
        if let Some(detail) = usage.detail.as_deref()
            && !detail.is_empty()
        {
            lines.push(String::new());
            lines.push(detail.to_string());
        }
    }
    if !profile.details.is_empty() {
        lines.push(String::new());
        lines.extend(profile.details.iter().cloned());
    } else if let Some(summary) = profile.error_summary.as_deref() {
        lines.push(String::new());
        lines.push(summary.to_string());
    } else {
        lines.push(String::new());
        lines.push("No detailed usage or warning data.".to_string());
    }
    lines.join("\n")
}

fn banner_style(kind: BannerKind) -> Style {
    if is_plain() {
        return Style::default().add_modifier(Modifier::BOLD);
    }
    match kind {
        BannerKind::Info => Style::default().fg(Color::Cyan),
        BannerKind::Success => Style::default().fg(Color::Green),
        BannerKind::Warning => Style::default().fg(Color::Yellow),
        BannerKind::Error => Style::default().fg(Color::Red),
    }
}

fn state_style(state: ProfileState) -> Style {
    if is_plain() {
        return Style::default().add_modifier(Modifier::BOLD);
    }
    match state {
        ProfileState::Ok => Style::default().fg(Color::Green),
        ProfileState::Warning => Style::default().fg(Color::Yellow),
        ProfileState::Unavailable => Style::default().fg(Color::Blue),
        ProfileState::Error => Style::default().fg(Color::Red),
    }
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars {
        return text.to_string();
    }
    chars[..max_chars.saturating_sub(1)]
        .iter()
        .collect::<String>()
        + "…"
}

fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let minutes = total / 60;
    let seconds = total % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_text_adds_ellipsis_only_when_needed() {
        assert_eq!(truncate_text("short", 10), "short");
        assert_eq!(truncate_text("abcdefgh", 5), "abcd…");
    }

    #[test]
    fn bucket_overview_formats_known_windows() {
        let bucket = crate::usage::UsageSnapshotBucket {
            id: "default".to_string(),
            label: "default".to_string(),
            five_hour: Some(crate::usage::UsageSnapshotWindow {
                left_percent: 88,
                reset_at: 0,
            }),
            weekly: Some(crate::usage::UsageSnapshotWindow {
                left_percent: 51,
                reset_at: 0,
            }),
        };

        assert_eq!(bucket_overview(&bucket).as_deref(), Some("5h 88% / 7d 51%"));
    }
}
