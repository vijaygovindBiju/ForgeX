use std::io;
use chrono::{Local, Utc};
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Tabs, Paragraph};

use forgex_core::{
    Commitment, CommitmentStatus, ForgeEvent, ForgeEventPayload,
    RecurrenceRule, Result as ForgeResult, SaveDay,
};
use forgex_scoring::{
    AvoidanceAnalyzer, PenaltyCalculator, RecoveryEngine,
};

use crate::app::AppContext;
use super::event::{poll_key_event, is_quit};
use super::ui::{dashboard, commitments, behavior, recovery, character, settings, forms};
use super::widgets;

/// Which top-level section is active
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Dashboard,
    Commitments,
    Behavior,
    Recovery,
    Character,
    Settings,
}

impl Section {
    pub const ALL: [Section; 6] = [
        Section::Dashboard,
        Section::Commitments,
        Section::Behavior,
        Section::Recovery,
        Section::Character,
        Section::Settings,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Section::Dashboard => "Dashboard",
            Section::Commitments => "Commitments",
            Section::Behavior => "Behavior",
            Section::Recovery => "Recovery",
            Section::Character => "Character",
            Section::Settings => "Settings",
        }
    }

    pub fn index(&self) -> usize {
        Section::ALL.iter().position(|s| s == self).unwrap_or(0)
    }
}

/// Overlay/dialog that can appear on top of a section
#[derive(Debug, Clone)]
pub enum Overlay {
    None,
    CreateCommitment(forms::CommitmentForm),
    LogActivity(forms::ActivityForm),
    HelpGuide,
    ConfirmAction {
        title: String,
        message: String,
        confirm_selected: bool,
        action: PendingAction,
    },
    StatusMessage {
        message: String,
        is_error: bool,
    },
}

/// Actions that need confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    StartCommitment(uuid::Uuid),
    CompleteCommitment(uuid::Uuid),
    MissCommitment(uuid::Uuid),
    DeleteCommitment(uuid::Uuid),
}

use std::sync::{Arc, RwLock};
use crate::watcher::{run_live_watcher, LiveWatcherInfo};

/// The main TUI application state
pub struct TuiApp {
    pub section: Section,
    pub overlay: Overlay,
    pub running: bool,

    // Commitments section state
    pub commitment_list_index: usize,
    pub commitment_filter: Option<CommitmentStatus>,

    // Behavior section state
    pub activity_scroll: u16,

    // Character section (no extra state needed)

    // Settings section
    pub settings_index: usize,
    pub settings_editing: Option<String>, // current editing buffer if editing a setting

    // Live background watcher state
    pub live_info: Arc<RwLock<Option<LiveWatcherInfo>>>,
}

impl TuiApp {
    pub fn new(live_info: Arc<RwLock<Option<LiveWatcherInfo>>>) -> Self {
        Self {
            section: Section::Dashboard,
            overlay: Overlay::None,
            running: true,
            commitment_list_index: 0,
            commitment_filter: None,
            activity_scroll: 0,
            settings_index: 0,
            settings_editing: None,
            live_info,
        }
    }
}

/// Main entry point: sets up terminal, runs the TUI loop, and restores terminal state.
pub fn run_tui(ctx: &mut AppContext) -> ForgeResult<()> {
    // Setup terminal
    enable_raw_mode()
        .map_err(|e| forgex_core::ForgeError::Config(format!("Failed to enable raw mode: {}", e)))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| forgex_core::ForgeError::Config(format!("Failed to enter alternate screen: {}", e)))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| forgex_core::ForgeError::Config(format!("Failed to create terminal: {}", e)))?;

    // Spawn automatic background watcher for live Hyprland / Wayland window tracking
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
    let live_info = Arc::new(RwLock::new(None));
    let watcher_handle = tokio::spawn(run_live_watcher(
        ctx.db.clone(),
        live_info.clone(),
        stop_rx,
    ));

    let mut app = TuiApp::new(live_info);
    let result = run_app(&mut terminal, &mut app, ctx);

    // Stop background watcher and flush remaining activity
    let _ = stop_tx.send(());
    // In synchronous context, do not block indefinitely; task exits on stop_rx
    let _ = watcher_handle;

    // Restore terminal regardless of error
    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
    ctx: &mut AppContext,
) -> ForgeResult<()> {
    while app.running {
        terminal.draw(|frame| draw(frame, app, ctx))
            .map_err(|e| forgex_core::ForgeError::Config(format!("Draw error: {}", e)))?;

        if let Some(key) = poll_key_event(100)
            .map_err(|e| forgex_core::ForgeError::Config(format!("Event error: {}", e)))? {

            if is_quit(&key) {
                app.running = false;
                continue;
            }

            handle_input(app, ctx, key);
        }
    }
    Ok(())
}

fn draw(frame: &mut Frame, app: &TuiApp, ctx: &AppContext) {
    let size = frame.area();

    // Layout: tab bar (3 lines) + content + footer (1 line)
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(size);

    // Tab bar with highlighted number badges
    let tab_titles: Vec<Line> = Section::ALL.iter().map(|s| {
        let key = s.index() + 1;
        let is_active = *s == app.section;
        let text_style = if is_active {
            Style::default().fg(Color::Black).bg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::White).bold()
        };
        let badge_style = if is_active {
            Style::default().fg(Color::Black).bg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::Yellow).bold()
        };
        Line::from(vec![
            Span::styled(format!(" [{}] ", key), badge_style),
            Span::styled(format!("{} ", s.label()), text_style),
        ])
    }).collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM).title(" ForgeX ").title_style(Style::default().fg(Color::Green).bold()))
        .select(app.section.index())
        .divider(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));

    frame.render_widget(tabs, chunks[0]);

    // Content area
    match app.section {
        Section::Dashboard => dashboard::render(frame, chunks[1], ctx),
        Section::Commitments => commitments::render(frame, chunks[1], app, ctx),
        Section::Behavior => behavior::render(frame, chunks[1], app, ctx),
        Section::Recovery => recovery::render(frame, chunks[1], ctx),
        Section::Character => character::render(frame, chunks[1], ctx),
        Section::Settings => settings::render(frame, chunks[1], app, ctx),
    }

    // High-visibility footer shortcuts
    let shortcuts = get_footer_shortcuts(app);
    widgets::render_footer_shortcuts(frame, chunks[2], &shortcuts);

    // Render overlay if present
    match &app.overlay {
        Overlay::None => {}
        Overlay::CreateCommitment(form) => {
            forms::render_commitment_form(frame, size, form);
        }
        Overlay::LogActivity(form) => {
            forms::render_activity_form(frame, size, form);
        }
        Overlay::HelpGuide => {
            forms::render_help_guide(frame, size);
        }
        Overlay::ConfirmAction {
            title,
            message,
            confirm_selected,
            ..
        } => {
            widgets::render_confirm_dialog(frame, size, title, message, *confirm_selected);
        }
        Overlay::StatusMessage { message, is_error } => {
            let color = if *is_error { Color::Red } else { Color::Green };
            let area = widgets::centered_rect_fixed(50, 5, size);
            frame.render_widget(ratatui::widgets::Clear, area);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color))
                .style(Style::default().bg(Color::Black));
            let inner = block.inner(area);
            frame.render_widget(block, area);
            let p = Paragraph::new(message.as_str())
                .alignment(Alignment::Center)
                .style(Style::default().fg(color));
            frame.render_widget(p, inner);
        }
    }
}

fn get_footer_shortcuts(app: &TuiApp) -> Vec<(&'static str, &'static str)> {
    match &app.overlay {
        Overlay::CreateCommitment(_) => vec![
            ("Tab", "Next Field"),
            ("Shift+Tab", "Prev"),
            ("↑/↓", "Rating/Rule"),
            ("Enter", "Create"),
            ("Esc", "Cancel"),
        ],
        Overlay::LogActivity(_) => vec![
            ("Tab", "Next Field"),
            ("↑/↓ / Space", "Category"),
            ("Enter", "Log Activity"),
            ("Esc", "Cancel"),
        ],
        Overlay::HelpGuide => vec![
            ("Esc / Enter / ?", "Close Guide"),
        ],
        Overlay::ConfirmAction { .. } => vec![
            ("←/→", "Select"),
            ("Enter", "Confirm"),
            ("Esc", "Cancel"),
        ],
        Overlay::StatusMessage { .. } => vec![
            ("Any Key", "Dismiss"),
        ],
        Overlay::None => match app.section {
            Section::Dashboard => vec![
                ("1-6", "Sections"),
                ("n", "New Task"),
                ("l", "Log Activity"),
                ("?", "User Guide"),
                ("q", "Quit"),
            ],
            Section::Commitments => vec![
                ("↑/↓", "Navigate"),
                ("n", "New"),
                ("s", "Start"),
                ("c", "Complete"),
                ("m", "Miss"),
                ("d", "Delete"),
                ("f", "Filter"),
                ("?", "Guide"),
                ("q", "Quit"),
            ],
            Section::Behavior => vec![
                ("↑/↓", "Scroll"),
                ("l", "Log Activity"),
                ("?", "Tracking Guide"),
                ("q", "Quit"),
            ],
            Section::Recovery => vec![
                ("1-6", "Sections"),
                ("?", "User Guide"),
                ("q", "Quit"),
            ],
            Section::Character => vec![
                ("1-6", "Sections"),
                ("?", "User Guide"),
                ("q", "Quit"),
            ],
            Section::Settings => vec![
                ("↑/↓", "Navigate"),
                ("Enter", "Edit Value"),
                ("?", "User Guide"),
                ("q", "Quit"),
            ],
        },
    }
}

fn handle_input(app: &mut TuiApp, ctx: &mut AppContext, key: crossterm::event::KeyEvent) {
    // Handle overlays first
    match &mut app.overlay {
        Overlay::StatusMessage { .. } => {
            app.overlay = Overlay::None;
            return;
        }
        Overlay::ConfirmAction {
            confirm_selected,
            action,
            ..
        } => {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    *confirm_selected = !*confirm_selected;
                }
                KeyCode::Enter => {
                    if *confirm_selected {
                        let action = action.clone();
                        app.overlay = Overlay::None;
                        execute_pending_action(app, ctx, action);
                    } else {
                        app.overlay = Overlay::None;
                    }
                }
                KeyCode::Esc => {
                    app.overlay = Overlay::None;
                }
                _ => {}
            }
            return;
        }
        Overlay::CreateCommitment(form) => {
            match key.code {
                KeyCode::Esc => {
                    app.overlay = Overlay::None;
                }
                KeyCode::Tab => {
                    form.next_field();
                }
                KeyCode::BackTab => {
                    form.prev_field();
                }
                KeyCode::Enter => {
                    if form.focused_field == forms::FormField::Submit {
                        let result = create_commitment_from_form(form, ctx);
                        match result {
                            Ok(msg) => {
                                app.overlay = Overlay::StatusMessage {
                                    message: msg,
                                    is_error: false,
                                };
                            }
                            Err(e) => {
                                app.overlay = Overlay::StatusMessage {
                                    message: format!("Error: {}", e),
                                    is_error: true,
                                };
                            }
                        }
                    } else {
                        form.next_field();
                    }
                }
                KeyCode::Char(c) => {
                    form.input_char(c);
                }
                KeyCode::Backspace => {
                    form.backspace();
                }
                KeyCode::Up => {
                    form.increment();
                }
                KeyCode::Down => {
                    form.decrement();
                }
                _ => {}
            }
            return;
        }
        Overlay::HelpGuide => {
            match key.code {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('h') => {
                    app.overlay = Overlay::None;
                }
                _ => {}
            }
            return;
        }
        Overlay::LogActivity(form) => {
            match key.code {
                KeyCode::Esc => {
                    app.overlay = Overlay::None;
                }
                KeyCode::Tab => {
                    form.next_field();
                }
                KeyCode::BackTab => {
                    form.prev_field();
                }
                KeyCode::Enter => {
                    if form.focused_field == forms::ActivityFormField::Submit {
                        let result = create_activity_from_form(form, ctx);
                        match result {
                            Ok(msg) => {
                                app.overlay = Overlay::StatusMessage {
                                    message: msg,
                                    is_error: false,
                                };
                            }
                            Err(e) => {
                                app.overlay = Overlay::StatusMessage {
                                    message: format!("Error: {}", e),
                                    is_error: true,
                                };
                            }
                        }
                    } else {
                        form.next_field();
                    }
                }
                KeyCode::Char(' ') => {
                    if form.focused_field == forms::ActivityFormField::Category {
                        form.toggle_category();
                    } else {
                        form.input_char(' ');
                    }
                }
                KeyCode::Up | KeyCode::Down => {
                    if form.focused_field == forms::ActivityFormField::Category {
                        form.toggle_category();
                    }
                }
                KeyCode::Char(c) => {
                    form.input_char(c);
                }
                KeyCode::Backspace => {
                    form.backspace();
                }
                _ => {}
            }
            return;
        }
        Overlay::None => {}
    }

    // Settings editing mode
    if app.section == Section::Settings {
        if let Some(ref mut buf) = app.settings_editing {
            match key.code {
                KeyCode::Esc => {
                    app.settings_editing = None;
                    return;
                }
                KeyCode::Enter => {
                    let value = buf.clone();
                    app.settings_editing = None;
                    apply_setting(app, ctx, value);
                    return;
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    return;
                }
                KeyCode::Backspace => {
                    buf.pop();
                    return;
                }
                _ => return,
            }
        }
    }

    // Global keys
    match key.code {
        KeyCode::Char('q') => {
            app.running = false;
            return;
        }
        KeyCode::Char('?') | KeyCode::Char('h') => {
            app.overlay = Overlay::HelpGuide;
            return;
        }
        KeyCode::Char('l') => {
            app.overlay = Overlay::LogActivity(forms::ActivityForm::new());
            return;
        }
        KeyCode::Char('1') => { app.section = Section::Dashboard; return; }
        KeyCode::Char('2') => { app.section = Section::Commitments; app.commitment_list_index = 0; return; }
        KeyCode::Char('3') => { app.section = Section::Behavior; app.activity_scroll = 0; return; }
        KeyCode::Char('4') => { app.section = Section::Recovery; return; }
        KeyCode::Char('5') => { app.section = Section::Character; return; }
        KeyCode::Char('6') => { app.section = Section::Settings; app.settings_index = 0; return; }
        _ => {}
    }

    // Section-specific keys
    match app.section {
        Section::Dashboard => handle_dashboard_input(app, ctx, key),
        Section::Commitments => handle_commitments_input(app, ctx, key),
        Section::Behavior => handle_behavior_input(app, key),
        Section::Recovery => {}
        Section::Character => {}
        Section::Settings => handle_settings_input(app, ctx, key),
    }
}

fn handle_dashboard_input(app: &mut TuiApp, _ctx: &AppContext, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Char('n') => {
            app.overlay = Overlay::CreateCommitment(forms::CommitmentForm::new());
        }
        _ => {}
    }
}

fn handle_commitments_input(app: &mut TuiApp, ctx: &AppContext, key: crossterm::event::KeyEvent) {
    let commitments = get_filtered_commitments(ctx, app.commitment_filter);
    let count = commitments.len();

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.commitment_list_index > 0 {
                app.commitment_list_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if count > 0 && app.commitment_list_index < count - 1 {
                app.commitment_list_index += 1;
            }
        }
        KeyCode::Char('n') => {
            app.overlay = Overlay::CreateCommitment(forms::CommitmentForm::new());
        }
        KeyCode::Char('s') => {
            if let Some(c) = commitments.get(app.commitment_list_index) {
                if c.status == CommitmentStatus::Planned {
                    app.overlay = Overlay::ConfirmAction {
                        title: "Start Commitment".to_string(),
                        message: format!("Start \"{}\"?", c.title),
                        confirm_selected: true,
                        action: PendingAction::StartCommitment(c.id),
                    };
                }
            }
        }
        KeyCode::Char('c') => {
            if let Some(c) = commitments.get(app.commitment_list_index) {
                if c.status == CommitmentStatus::Planned || c.status == CommitmentStatus::Active {
                    app.overlay = Overlay::ConfirmAction {
                        title: "Complete Commitment".to_string(),
                        message: format!("Mark \"{}\" as completed?", c.title),
                        confirm_selected: true,
                        action: PendingAction::CompleteCommitment(c.id),
                    };
                }
            }
        }
        KeyCode::Char('m') => {
            if let Some(c) = commitments.get(app.commitment_list_index) {
                if c.status == CommitmentStatus::Planned || c.status == CommitmentStatus::Active {
                    app.overlay = Overlay::ConfirmAction {
                        title: "Miss Commitment".to_string(),
                        message: format!("Mark \"{}\" as missed?\nThis may calculate a penalty based on your recorded behavior.", c.title),
                        confirm_selected: false, // default to Cancel for dangerous action
                        action: PendingAction::MissCommitment(c.id),
                    };
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(c) = commitments.get(app.commitment_list_index) {
                app.overlay = Overlay::ConfirmAction {
                    title: "Delete Commitment".to_string(),
                    message: format!("Permanently delete \"{}\"?", c.title),
                    confirm_selected: false,
                    action: PendingAction::DeleteCommitment(c.id),
                };
            }
        }
        KeyCode::Char('f') => {
            // Cycle filter: None -> Planned -> Active -> Completed -> Missed -> None
            app.commitment_filter = match app.commitment_filter {
                None => Some(CommitmentStatus::Planned),
                Some(CommitmentStatus::Planned) => Some(CommitmentStatus::Active),
                Some(CommitmentStatus::Active) => Some(CommitmentStatus::Completed),
                Some(CommitmentStatus::Completed) => Some(CommitmentStatus::Missed),
                Some(CommitmentStatus::Missed) => None,
                Some(_) => None,
            };
            app.commitment_list_index = 0;
        }
        _ => {}
    }
}

fn handle_behavior_input(app: &mut TuiApp, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.activity_scroll = app.activity_scroll.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.activity_scroll = app.activity_scroll.saturating_add(1);
        }
        KeyCode::Char('l') => {
            app.overlay = Overlay::LogActivity(forms::ActivityForm::new());
        }
        _ => {}
    }
}

fn handle_settings_input(app: &mut TuiApp, _ctx: &AppContext, key: crossterm::event::KeyEvent) {
    let setting_count = 6usize; // number of config fields
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            if app.settings_index > 0 {
                app.settings_index -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.settings_index < setting_count - 1 {
                app.settings_index += 1;
            }
        }
        KeyCode::Enter => {
            // Start editing: pre-fill with current value
            let current = get_setting_value_str(_ctx, app.settings_index);
            app.settings_editing = Some(current);
        }
        _ => {}
    }
}

fn apply_setting(app: &mut TuiApp, ctx: &mut AppContext, value: String) {
    let result = match app.settings_index {
        0 => value.parse::<u32>().map(|v| ctx.config.max_daily_penalty_mins = v),
        1 => value.parse::<u32>().map(|v| ctx.config.save_days_max = v),
        2 => value.parse::<u32>().map(|v| ctx.config.save_day_streak_required = v),
        3 => value.parse::<u8>().map(|v| ctx.config.auto_save_day_importance_threshold = v.clamp(1, 5)),
        4 => value.parse::<u32>().map(|v| ctx.config.default_entertainment_allowance_mins = v),
        5 => value.parse::<u32>().map(|v| ctx.config.grace_period_mins = v),
        _ => Ok(()),
    };

    match result {
        Ok(()) => {
            if let Err(e) = ctx.save_config() {
                app.overlay = Overlay::StatusMessage {
                    message: format!("Failed to save: {}", e),
                    is_error: true,
                };
            } else {
                app.overlay = Overlay::StatusMessage {
                    message: "Setting updated.".to_string(),
                    is_error: false,
                };
            }
        }
        Err(_) => {
            app.overlay = Overlay::StatusMessage {
                message: "Invalid value.".to_string(),
                is_error: true,
            };
        }
    }
}

fn get_setting_value_str(ctx: &AppContext, index: usize) -> String {
    match index {
        0 => ctx.config.max_daily_penalty_mins.to_string(),
        1 => ctx.config.save_days_max.to_string(),
        2 => ctx.config.save_day_streak_required.to_string(),
        3 => ctx.config.auto_save_day_importance_threshold.to_string(),
        4 => ctx.config.default_entertainment_allowance_mins.to_string(),
        5 => ctx.config.grace_period_mins.to_string(),
        _ => String::new(),
    }
}

// ─── Domain actions ─────────────────────────────────────────────

fn execute_pending_action(app: &mut TuiApp, ctx: &mut AppContext, action: PendingAction) {
    let result = match action {
        PendingAction::StartCommitment(id) => action_start_commitment(ctx, id),
        PendingAction::CompleteCommitment(id) => action_complete_commitment(ctx, id),
        PendingAction::MissCommitment(id) => action_miss_commitment(ctx, id),
        PendingAction::DeleteCommitment(id) => action_delete_commitment(ctx, id),
    };

    match result {
        Ok(msg) => {
            app.overlay = Overlay::StatusMessage {
                message: msg,
                is_error: false,
            };
        }
        Err(e) => {
            app.overlay = Overlay::StatusMessage {
                message: format!("Error: {}", e),
                is_error: true,
            };
        }
    }
}

fn action_start_commitment(ctx: &AppContext, id: uuid::Uuid) -> ForgeResult<String> {
    let mut c = ctx.db.get_commitment(id)?
        .ok_or_else(|| forgex_core::ForgeError::NotFound("Commitment not found".into()))?;
    let now = Utc::now();
    c.start(now)?;
    ctx.db.update_commitment(&c)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskStarted {
        commitment_id: c.id,
    }))?;
    Ok(format!("Started: {}", c.title))
}

fn action_complete_commitment(ctx: &AppContext, id: uuid::Uuid) -> ForgeResult<String> {
    let mut c = ctx.db.get_commitment(id)?
        .ok_or_else(|| forgex_core::ForgeError::NotFound("Commitment not found".into()))?;
    let now = Utc::now();
    c.complete(now)?;
    ctx.db.update_commitment(&c)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCompleted {
        commitment_id: c.id,
    }))?;

    let mut msg = format!("Completed: {}", c.title);

    // Apply recovery
    let active_penalties = ctx.db.get_active_penalties()?;
    let assessment = RecoveryEngine::evaluate_task_completion(&c, active_penalties.first());

    if let Some(mut penalty) = active_penalties.into_iter().next() {
        let reduction = assessment.penalty_reduction_mins;
        penalty.reduce(reduction, now);
        ctx.db.update_penalty(&penalty)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::PenaltyReduced {
            penalty_id: penalty.id,
            reduced_by_mins: reduction,
            remaining_mins: penalty.remaining_penalty_mins,
        }))?;
        msg.push_str(&format!("\n⚡ Recovered {}. Remaining: {}",
            widgets::fmt_duration(reduction),
            widgets::fmt_duration(penalty.remaining_penalty_mins)));

        if penalty.remaining_penalty_mins == 0 {
            msg.push_str("\n🎉 All penalties cleared!");
        }
    }

    // Screen time bonus
    let bonus = assessment.screen_time_bonus_mins;
    if bonus > 0 {
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::ScreenTimeGranted {
            minutes: bonus,
        }))?;
        msg.push_str(&format!("\n⏱️ Earned +{} entertainment allowance", widgets::fmt_duration(bonus)));
    }

    // Check Save Day earning
    let all_commitments = ctx.db.list_commitments()?;
    let completed_streak = all_commitments.iter().filter(|cc| cc.status == CommitmentStatus::Completed).count() as u32;
    let available_save_days = ctx.db.get_available_save_days()?.len() as u32;
    if RecoveryEngine::should_earn_save_day(available_save_days, completed_streak, &ctx.config) {
        let new_save_day = SaveDay::new_earned(now);
        ctx.db.insert_save_day(&new_save_day)?;
        ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::SaveDayEarned {
            save_day_id: new_save_day.id,
        }))?;
        msg.push_str(&format!("\n🛡️ Earned a Save Day! (Balance: {})", available_save_days + 1));
    }

    Ok(msg)
}

fn action_miss_commitment(ctx: &AppContext, id: uuid::Uuid) -> ForgeResult<String> {
    let mut c = ctx.db.get_commitment(id)?
        .ok_or_else(|| forgex_core::ForgeError::NotFound("Commitment not found".into()))?;
    let now = Utc::now();
    c.miss(now)?;
    ctx.db.update_commitment(&c)?;

    // Avoidance
    let window_start = c.scheduled_start - chrono::Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let window_end = c.scheduled_end() + chrono::Duration::minutes(AvoidanceAnalyzer::WINDOW_BUFFER_MINS);
    let activities = ctx.db.list_activities_for_window(window_start, window_end)?;
    let avoidance = AvoidanceAnalyzer::assess(&c, &activities);

    // Penalty calc
    let active_penalties = ctx.db.get_active_penalties()?;
    let current_restricted = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);
    let prev_hit_max = active_penalties.iter().any(|p| p.actual_penalty_mins >= ctx.config.max_daily_penalty_mins);

    let calc = PenaltyCalculator::calculate(&c, &avoidance, &ctx.config, current_restricted, prev_hit_max);

    // Save Day shielding
    let available_save_days = ctx.db.get_available_save_days()?;
    let should_shield = RecoveryEngine::should_shield_missed_task(
        &c,
        available_save_days.len() as u32,
        &ctx.config,
        false,
        false,
    );

    if should_shield {
        ctx.db.consume_oldest_save_day(c.id, now)?;
    }

    let penalty_record = PenaltyCalculator::to_penalty_record(&c, &calc, should_shield);
    ctx.db.insert_penalty(&penalty_record)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskMissed {
        commitment_id: c.id,
        penalty_mins: penalty_record.actual_penalty_mins,
        shielded: should_shield,
    }))?;

    let mut msg = format!("Missed: {}", c.title);
    if should_shield {
        msg.push_str("\n🛡️ Save Day shielded this failure! No penalty applied.");
    } else {
        msg.push_str(&format!("\n⚠️ Penalty: {} restriction applied",
            widgets::fmt_duration(penalty_record.actual_penalty_mins)));
        if penalty_record.restricted_days > 0 {
            msg.push_str(&format!("\nRestricted days: {}", penalty_record.restricted_days));
        }
    }

    Ok(msg)
}

fn action_delete_commitment(ctx: &AppContext, id: uuid::Uuid) -> ForgeResult<String> {
    let c = ctx.db.get_commitment(id)?
        .ok_or_else(|| forgex_core::ForgeError::NotFound("Commitment not found".into()))?;
    ctx.db.delete_commitment(id)?;
    Ok(format!("Deleted: {}", c.title))
}

fn create_commitment_from_form(form: &forms::CommitmentForm, ctx: &AppContext) -> ForgeResult<String> {
    let title = form.title.trim();
    if title.is_empty() {
        return Err(forgex_core::ForgeError::Validation("Title cannot be empty".into()));
    }

    let duration: u32 = form.duration.trim().parse()
        .map_err(|_| forgex_core::ForgeError::Validation("Duration must be a number".into()))?;

    let scheduled_start = if form.schedule.trim().is_empty() {
        Utc::now()
    } else {
        crate::commands::task::parse_schedule_time(form.schedule.trim())?
    };

    let recurrence = match form.recurrence.trim().to_lowercase().as_str() {
        "once" => RecurrenceRule::Once,
        "weekdays" => RecurrenceRule::Weekdays,
        _ => RecurrenceRule::Daily,
    };

    let mut commitment = Commitment::new(
        title,
        form.importance,
        form.effort,
        scheduled_start,
        duration,
        recurrence,
        if form.category.trim().is_empty() { "General" } else { form.category.trim() },
    )?;

    if !form.description.trim().is_empty() {
        commitment.description = Some(form.description.trim().to_string());
    }

    ctx.db.insert_commitment(&commitment)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::TaskCreated {
        commitment_id: commitment.id,
        title: commitment.title.clone(),
    }))?;

    Ok(format!("Created: {} (I:{}/5 E:{}/5)", commitment.title, commitment.importance, commitment.expected_effort))
}

fn create_activity_from_form(form: &forms::ActivityForm, ctx: &AppContext) -> ForgeResult<String> {
    let app_name = form.application.trim();
    if app_name.is_empty() {
        return Err(forgex_core::ForgeError::Validation("Application name cannot be empty".into()));
    }

    let duration: u32 = form.duration.trim().parse()
        .map_err(|_| forgex_core::ForgeError::Validation("Duration must be a positive integer in minutes".into()))?;

    let started_at = Utc::now() - chrono::Duration::minutes(duration as i64);
    let detail = if form.detail.trim().is_empty() {
        None
    } else {
        Some(form.detail.trim().to_string())
    };

    let act = forgex_core::Activity::new(
        "linux-local",
        app_name,
        detail,
        form.category,
        started_at,
        duration,
    )?;

    ctx.db.insert_activity(&act)?;
    ctx.db.log_event(&ForgeEvent::new(ForgeEventPayload::ActivityLogged {
        activity_id: act.id,
        application: act.application.clone(),
        duration_mins: act.duration_mins,
    }))?;

    Ok(format!("Logged activity: {} ({}) for {}m", act.application, act.category, act.duration_mins))
}

// ─── Helpers used by UI renderers ────────────────────────────────

pub fn get_filtered_commitments(ctx: &AppContext, filter: Option<CommitmentStatus>) -> Vec<Commitment> {
    let all = ctx.db.list_commitments().unwrap_or_default();
    match filter {
        None => all,
        Some(status) => all.into_iter().filter(|c| c.status == status).collect(),
    }
}

pub fn get_today_commitments(ctx: &AppContext) -> Vec<Commitment> {
    let today = Local::now().date_naive();
    ctx.db.list_commitments().unwrap_or_default()
        .into_iter()
        .filter(|c| c.scheduled_start.with_timezone(&Local).date_naive() == today)
        .collect()
}
