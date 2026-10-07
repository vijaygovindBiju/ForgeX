use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use forgex_scoring::CharacterCalculator;
use crate::app::AppContext;
use crate::tui::widgets::progress_str;

pub fn render(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let block = Block::default()
        .title(" Character Profile ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let commitments = ctx.db.list_commitments().unwrap_or_default();
    let penalties = ctx.db.list_penalties().unwrap_or_default();
    let profile = CharacterCalculator::calculate(&commitments, &penalties);

    let avg = (profile.consistency
        + profile.reliability
        + profile.resilience
        + profile.self_control
        + profile.discipline)
        / 5;

    let chunks = Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(12),
        Constraint::Min(2),
    ])
    .split(inner);

    // Header info
    let header_lines = vec![
        Line::from(vec![
            Span::raw("  Mastery Level: "),
            Span::styled(
                format!("Level {}", profile.overall_level),
                Style::default().fg(Color::Cyan).bold(),
            ),
            Span::raw("  "),
            Span::styled(format!("(Overall Average: {}%)", avg), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(Span::styled(
            "  Character metrics reflect observed behavior and consistency over time.",
            Style::default().fg(Color::DarkGray).italic(),
        )),
    ];
    frame.render_widget(Paragraph::new(header_lines), chunks[0]);

    // The 5 pillars
    let mut metric_lines = Vec::new();
    add_metric_line(&mut metric_lines, "Consistency", profile.consistency, "Habit completion frequency");
    add_metric_line(&mut metric_lines, "Reliability", profile.reliability, "Completing without prior skips");
    add_metric_line(&mut metric_lines, "Resilience", profile.resilience, "Recovery rate after missed commitments");
    add_metric_line(&mut metric_lines, "Self-Control", profile.self_control, "Absence of entertainment during tasks");
    add_metric_line(&mut metric_lines, "Discipline", profile.discipline, "Follow-through on high effort/importance");

    frame.render_widget(Paragraph::new(metric_lines), chunks[1]);

    // Footer guidance
    let footer_text = vec![
        Line::from(Span::styled(
            "  💡 Character is forged through repeated follow-through.",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "  Complete high-effort commitments to increase Discipline and Reliability.",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(footer_text), chunks[2]);
}

fn add_metric_line(lines: &mut Vec<Line>, name: &str, score: u32, description: &str) {
    let color = if score >= 80 {
        Color::Green
    } else if score >= 60 {
        Color::Yellow
    } else {
        Color::Red
    };

    lines.push(Line::from(vec![
        Span::styled(format!("  {:<14} ", name), Style::default().bold()),
        Span::styled(format!("{:>3}% ", score), Style::default().fg(color).bold()),
        Span::styled(format!("[{}] ", progress_str(score, 100, 20)), Style::default().fg(color)),
        Span::styled(format!(" {}", description), Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));
}
