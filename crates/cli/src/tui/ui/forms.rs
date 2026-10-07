use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use forgex_core::ActivityCategory;
use crate::tui::widgets::centered_rect_fixed;

// ============================================================================
// Commitment Form
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormField {
    Title,
    Duration,
    Importance,
    Effort,
    Schedule,
    Recurrence,
    Category,
    Description,
    Submit,
}

#[derive(Debug, Clone)]
pub struct CommitmentForm {
    pub title: String,
    pub duration: String,
    pub importance: u8,
    pub effort: u8,
    pub schedule: String,
    pub recurrence: String,
    pub category: String,
    pub description: String,
    pub focused_field: FormField,
}

impl CommitmentForm {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            duration: "45".to_string(),
            importance: 3,
            effort: 3,
            schedule: String::new(),
            recurrence: "daily".to_string(),
            category: "General".to_string(),
            description: String::new(),
            focused_field: FormField::Title,
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            FormField::Title => FormField::Duration,
            FormField::Duration => FormField::Importance,
            FormField::Importance => FormField::Effort,
            FormField::Effort => FormField::Schedule,
            FormField::Schedule => FormField::Recurrence,
            FormField::Recurrence => FormField::Category,
            FormField::Category => FormField::Description,
            FormField::Description => FormField::Submit,
            FormField::Submit => FormField::Title,
        };
    }

    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            FormField::Title => FormField::Submit,
            FormField::Duration => FormField::Title,
            FormField::Importance => FormField::Duration,
            FormField::Effort => FormField::Importance,
            FormField::Schedule => FormField::Effort,
            FormField::Recurrence => FormField::Schedule,
            FormField::Category => FormField::Recurrence,
            FormField::Description => FormField::Category,
            FormField::Submit => FormField::Description,
        };
    }

    pub fn input_char(&mut self, c: char) {
        match self.focused_field {
            FormField::Title => self.title.push(c),
            FormField::Duration => {
                if c.is_ascii_digit() {
                    self.duration.push(c);
                }
            }
            FormField::Importance => {
                if let Some(digit) = c.to_digit(10) {
                    if (1..=5).contains(&digit) {
                        self.importance = digit as u8;
                    }
                }
            }
            FormField::Effort => {
                if let Some(digit) = c.to_digit(10) {
                    if (1..=5).contains(&digit) {
                        self.effort = digit as u8;
                    }
                }
            }
            FormField::Schedule => self.schedule.push(c),
            FormField::Recurrence => self.recurrence.push(c),
            FormField::Category => self.category.push(c),
            FormField::Description => self.description.push(c),
            FormField::Submit => {}
        }
    }

    pub fn backspace(&mut self) {
        match self.focused_field {
            FormField::Title => { self.title.pop(); }
            FormField::Duration => { self.duration.pop(); }
            FormField::Schedule => { self.schedule.pop(); }
            FormField::Recurrence => { self.recurrence.pop(); }
            FormField::Category => { self.category.pop(); }
            FormField::Description => { self.description.pop(); }
            _ => {}
        }
    }

    pub fn increment(&mut self) {
        match self.focused_field {
            FormField::Importance => {
                if self.importance < 5 {
                    self.importance += 1;
                }
            }
            FormField::Effort => {
                if self.effort < 5 {
                    self.effort += 1;
                }
            }
            FormField::Recurrence => {
                self.recurrence = match self.recurrence.to_lowercase().as_str() {
                    "daily" => "weekdays".to_string(),
                    "weekdays" => "once".to_string(),
                    _ => "daily".to_string(),
                };
            }
            _ => {}
        }
    }

    pub fn decrement(&mut self) {
        match self.focused_field {
            FormField::Importance => {
                if self.importance > 1 {
                    self.importance -= 1;
                }
            }
            FormField::Effort => {
                if self.effort > 1 {
                    self.effort -= 1;
                }
            }
            FormField::Recurrence => {
                self.recurrence = match self.recurrence.to_lowercase().as_str() {
                    "daily" => "once".to_string(),
                    "once" => "weekdays".to_string(),
                    _ => "daily".to_string(),
                };
            }
            _ => {}
        }
    }
}

pub fn render_commitment_form(frame: &mut Frame, area: Rect, form: &CommitmentForm) {
    let dialog_area = centered_rect_fixed(62, 21, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Create Commitment ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).bold())
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::vertical([
        Constraint::Length(2), // Title
        Constraint::Length(2), // Duration
        Constraint::Length(2), // Importance
        Constraint::Length(2), // Effort
        Constraint::Length(2), // Schedule
        Constraint::Length(2), // Recurrence
        Constraint::Length(2), // Category
        Constraint::Length(2), // Description
        Constraint::Length(2), // Buttons
    ])
    .split(inner);

    render_field_row(frame, rows[0], "Title:", &form.title, form.focused_field == FormField::Title, "What do you commit to?");
    render_field_row(frame, rows[1], "Duration (min):", &form.duration, form.focused_field == FormField::Duration, "e.g. 30, 45, 60");
    render_rating_row(frame, rows[2], "Importance:", form.importance, form.focused_field == FormField::Importance);
    render_rating_row(frame, rows[3], "Expected Effort:", form.effort, form.focused_field == FormField::Effort);
    render_field_row(frame, rows[4], "Scheduled Time:", &form.schedule, form.focused_field == FormField::Schedule, "e.g. 19:00, +30m, empty for now");
    render_field_row(frame, rows[5], "Recurrence:", &form.recurrence, form.focused_field == FormField::Recurrence, "daily, weekdays, once (↑/↓ to toggle)");
    render_field_row(frame, rows[6], "Category:", &form.category, form.focused_field == FormField::Category, "e.g. Work, Health, Study");
    render_field_row(frame, rows[7], "Description:", &form.description, form.focused_field == FormField::Description, "optional notes");

    // Submit button
    let submit_style = if form.focused_field == FormField::Submit {
        Style::default().fg(Color::Black).bg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Green).bold()
    };

    let cancel_style = Style::default().fg(Color::White);

    let btns = Line::from(vec![
        Span::styled("   [ Create Commitment ]   ", submit_style),
        Span::raw("   "),
        Span::styled("   [ Esc to Cancel ]   ", cancel_style),
    ]);
    frame.render_widget(Paragraph::new(btns).alignment(Alignment::Center), rows[8]);
}

// ============================================================================
// Activity Form (Manual Activity Logging in TUI)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityFormField {
    Application,
    Detail,
    Duration,
    Category,
    Submit,
}

#[derive(Debug, Clone)]
pub struct ActivityForm {
    pub application: String,
    pub detail: String,
    pub duration: String,
    pub category: ActivityCategory,
    pub focused_field: ActivityFormField,
}

impl ActivityForm {
    pub fn new() -> Self {
        Self {
            application: String::new(),
            detail: String::new(),
            duration: "30".to_string(),
            category: ActivityCategory::Productive,
            focused_field: ActivityFormField::Application,
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = match self.focused_field {
            ActivityFormField::Application => ActivityFormField::Detail,
            ActivityFormField::Detail => ActivityFormField::Duration,
            ActivityFormField::Duration => ActivityFormField::Category,
            ActivityFormField::Category => ActivityFormField::Submit,
            ActivityFormField::Submit => ActivityFormField::Application,
        };
    }

    pub fn prev_field(&mut self) {
        self.focused_field = match self.focused_field {
            ActivityFormField::Application => ActivityFormField::Submit,
            ActivityFormField::Detail => ActivityFormField::Application,
            ActivityFormField::Duration => ActivityFormField::Detail,
            ActivityFormField::Category => ActivityFormField::Duration,
            ActivityFormField::Submit => ActivityFormField::Category,
        };
    }

    pub fn input_char(&mut self, c: char) {
        match self.focused_field {
            ActivityFormField::Application => self.application.push(c),
            ActivityFormField::Detail => self.detail.push(c),
            ActivityFormField::Duration => {
                if c.is_ascii_digit() {
                    self.duration.push(c);
                }
            }
            ActivityFormField::Category => {}
            ActivityFormField::Submit => {}
        }
    }

    pub fn backspace(&mut self) {
        match self.focused_field {
            ActivityFormField::Application => { self.application.pop(); }
            ActivityFormField::Detail => { self.detail.pop(); }
            ActivityFormField::Duration => { self.duration.pop(); }
            _ => {}
        }
    }

    pub fn toggle_category(&mut self) {
        self.category = match self.category {
            ActivityCategory::Productive => ActivityCategory::Essential,
            ActivityCategory::Essential => ActivityCategory::MediumEntertainment,
            ActivityCategory::MediumEntertainment => ActivityCategory::HighEntertainment,
            ActivityCategory::HighEntertainment => ActivityCategory::Productive,
            ActivityCategory::Unknown => ActivityCategory::Productive,
        };
    }
}

pub fn render_activity_form(frame: &mut Frame, area: Rect, form: &ActivityForm) {
    let dialog_area = centered_rect_fixed(60, 15, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" Log Activity / Behavioral Record ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow).bold())
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let rows = Layout::vertical([
        Constraint::Length(2), // App
        Constraint::Length(2), // Detail
        Constraint::Length(2), // Duration
        Constraint::Length(2), // Category
        Constraint::Length(2), // Submit
    ])
    .split(inner);

    render_field_row(frame, rows[0], "Application:", &form.application, form.focused_field == ActivityFormField::Application, "e.g. VS Code, Firefox, Steam");
    render_field_row(frame, rows[1], "Detail / URL:", &form.detail, form.focused_field == ActivityFormField::Detail, "e.g. youtube.com, rust-lang.org");
    render_field_row(frame, rows[2], "Duration (min):", &form.duration, form.focused_field == ActivityFormField::Duration, "e.g. 15, 30, 60");

    // Category row
    let cat_color = match form.category {
        ActivityCategory::Productive => Color::Green,
        ActivityCategory::Essential => Color::Cyan,
        ActivityCategory::MediumEntertainment => Color::Yellow,
        ActivityCategory::HighEntertainment => Color::Red,
        ActivityCategory::Unknown => Color::DarkGray,
    };
    let cat_focused = form.focused_field == ActivityFormField::Category;
    let cat_line = Line::from(vec![
        Span::styled(" Category:          ", if cat_focused { Style::default().fg(Color::Cyan).bold() } else { Style::default().fg(Color::White) }),
        Span::styled(format!("[ {} ]", form.category.to_string()), Style::default().fg(cat_color).bold()),
        if cat_focused {
            Span::styled("  (↑/↓ or Space to toggle)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(cat_line), rows[3]);

    // Submit buttons
    let submit_style = if form.focused_field == ActivityFormField::Submit {
        Style::default().fg(Color::Black).bg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Green).bold()
    };
    let cancel_style = Style::default().fg(Color::White);

    let btns = Line::from(vec![
        Span::styled("   [ Log Activity ]   ", submit_style),
        Span::raw("   "),
        Span::styled("   [ Esc to Cancel ]   ", cancel_style),
    ]);
    frame.render_widget(Paragraph::new(btns).alignment(Alignment::Center), rows[4]);
}

// ============================================================================
// Comprehensive User Guide Modal (Visible via '?' or 'h')
// ============================================================================

pub fn render_help_guide(frame: &mut Frame, area: Rect) {
    let dialog_area = centered_rect_fixed(74, 25, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(" ForgeX — Keyboard Shortcuts & User Guide ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow).bold())
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let guide_text = vec![
        Line::from(vec![
            Span::styled("🎯 TOP NAVIGATION:", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::styled("  1-6  ", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
            Span::raw(" Jump between sections: [1]Dashboard [2]Commitments [3]Behavior [4]Recovery [5]Character [6]Settings"),
        ]),
        Line::from(vec![
            Span::styled("  ? / h", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
            Span::raw(" Open this full User Guide"),
            Span::raw("   "),
            Span::styled("  q    ", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
            Span::raw(" Quit ForgeX safely"),
        ]),
        Line::from(""),

        Line::from(vec![
            Span::styled("📋 COMMITMENTS TAB (Press '2'):", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓, k/j ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Select task      "),
            Span::styled("  n ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" New commitment form"),
        ]),
        Line::from(vec![
            Span::styled("  s        ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Start task (Active)  "),
            Span::styled("  c ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Mark Complete (earns recovery points)"),
        ]),
        Line::from(vec![
            Span::styled("  m        ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Mark Missed (penalty)"),
            Span::styled("  d ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Delete task    "),
            Span::styled("  f ", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" Cycle status filter"),
        ]),
        Line::from(""),

        Line::from(vec![
            Span::styled("👁️ ACTIVITY TRACKING — WHY IS NOTHING TRACKING AUTOMATICALLY?", Style::default().fg(Color::Magenta).bold()),
        ]),
        Line::from(vec![
            Span::styled("  • Automatic Tracking:", Style::default().fg(Color::Green).bold()),
            Span::raw(" Requires the background watcher daemon."),
        ]),
        Line::from(vec![
            Span::raw("    In another terminal, run: "),
            Span::styled("forgex watch", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" (monitors Hyprland/Wayland/X11 active windows)."),
        ]),
        Line::from(vec![
            Span::styled("  • Manual Tracking:   ", Style::default().fg(Color::Green).bold()),
            Span::raw(" Press "),
            Span::styled("l", Style::default().fg(Color::Yellow).bold()),
            Span::raw(" in the TUI to log work or entertainment right now."),
        ]),
        Line::from(vec![
            Span::raw("    Avoidance score is calculated from entertainment overlapping commitment windows."),
        ]),
        Line::from(""),

        Line::from(vec![
            Span::styled("🛡️ RECOVERY & SAVE DAYS:", Style::default().fg(Color::Cyan).bold()),
        ]),
        Line::from(vec![
            Span::raw("  • Completing tasks clears penalties (15m penalty recovered per effort point)."),
        ]),
        Line::from(vec![
            Span::raw("  • Every 7-day consistency streak awards 1 "),
            Span::styled("Save Day", Style::default().fg(Color::Green).bold()),
            Span::raw(" to automatically shield accidental misses."),
        ]),
        Line::from(""),

        Line::from(vec![
            Span::styled("                [ Press Esc, Enter, or '?' to close this guide ]", Style::default().fg(Color::Yellow).bold()),
        ]),
    ];

    frame.render_widget(Paragraph::new(guide_text), inner);
}

// ============================================================================
// Internal Helpers
// ============================================================================

fn render_field_row(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool, hint: &str) {
    let style = if focused {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let display_val = if value.is_empty() && !focused {
        format!("[ {} ]", hint)
    } else {
        format!("[ {}{} ]", value, if focused { "_" } else { "" })
    };

    let val_style = if value.is_empty() && !focused {
        Style::default().fg(Color::DarkGray)
    } else if focused {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let line = Line::from(vec![
        Span::styled(format!(" {:<18} ", label), style),
        Span::styled(display_val, val_style),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_rating_row(frame: &mut Frame, area: Rect, label: &str, val: u8, focused: bool) {
    let style = if focused {
        Style::default().fg(Color::Cyan).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let val_style = if focused {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };

    let stars = "★".repeat(val as usize);
    let empty = "☆".repeat(5 - val as usize);

    let line = Line::from(vec![
        Span::styled(format!(" {:<18} ", label), style),
        Span::styled(format!("[ {} / 5 ]  {}{}", val, stars, empty), val_style),
        if focused {
            Span::styled("  (1-5 or ↑/↓)", Style::default().fg(Color::DarkGray))
        } else {
            Span::raw("")
        },
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
