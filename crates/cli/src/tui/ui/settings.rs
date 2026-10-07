use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::app::AppContext;
use crate::tui::app::TuiApp;

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp, ctx: &AppContext) {
    let block = Block::default()
        .title(" System Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(3),  // Config file path
        Constraint::Length(14), // Settings list
        Constraint::Min(2),     // Guidance
    ])
    .split(inner);

    // Path
    let path_line = vec![
        Line::from(vec![
            Span::raw("  Configuration File: "),
            Span::styled(ctx.config_path.display().to_string(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(Span::styled(
            "  Select a setting with ↑/↓ and press Enter to edit. Enter saves, Esc cancels.",
            Style::default().fg(Color::DarkGray).italic(),
        )),
    ];
    frame.render_widget(Paragraph::new(path_line), chunks[0]);

    // Settings list
    let settings = [
        ("Max Daily Penalty", format!("{} mins", ctx.config.max_daily_penalty_mins), "Daily cap for calculated penalty minutes"),
        ("Max Save Days", format!("{}", ctx.config.save_days_max), "Maximum bankable Save Day tokens"),
        ("Save Day Streak Required", format!("{} days", ctx.config.save_day_streak_required), "Consecutive successful days needed to earn 1 Save Day"),
        ("Auto Save Day Threshold", format!("{}/5 importance", ctx.config.auto_save_day_importance_threshold), "Minimum importance required to auto-shield a missed task"),
        ("Entertainment Allowance", format!("{} mins", ctx.config.default_entertainment_allowance_mins), "Base daily entertainment screen-time allowance"),
        ("Grace Period", format!("{} mins", ctx.config.grace_period_mins), "Buffer after scheduled end before task is marked overdue"),
    ];

    let mut lines = Vec::new();
    for (i, (name, val, desc)) in settings.iter().enumerate() {
        let is_selected = i == app.settings_index;
        let is_editing = is_selected && app.settings_editing.is_some();

        let prefix = if is_selected { " ▸ " } else { "   " };
        let name_style = if is_selected {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::White)
        };

        let val_display = if is_editing {
            format!("[ {}_ ]", app.settings_editing.as_ref().unwrap())
        } else {
            format!("[ {} ]", val)
        };

        let val_style = if is_editing {
            Style::default().fg(Color::Yellow).bold()
        } else if is_selected {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:<32}", name), name_style),
            Span::styled(format!("{:<20}", val_display), val_style),
            Span::styled(format!(" {}", desc), Style::default().fg(Color::DarkGray)),
        ]));
        lines.push(Line::from(""));
    }

    frame.render_widget(Paragraph::new(lines), chunks[1]);

    // Guidance footer
    let help = vec![
        Line::from(Span::styled(
            "  ⚙ Settings changes are validated and saved directly to disk.",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(help), chunks[2]);
}
