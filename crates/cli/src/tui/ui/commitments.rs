use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Row, Table, Cell};

use forgex_core::CommitmentStatus;
use crate::app::AppContext;
use crate::tui::app::{TuiApp, get_filtered_commitments};
use crate::tui::widgets::fmt_duration;

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp, ctx: &AppContext) {
    let commitments = get_filtered_commitments(ctx, app.commitment_filter);

    let filter_label = match app.commitment_filter {
        None => "All".to_string(),
        Some(s) => s.to_string(),
    };

    let block = Block::default()
        .title(format!(" Commitments [{}] — press 'f' to filter ", filter_label))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if commitments.is_empty() {
        let msg = ratatui::widgets::Paragraph::new("  No commitments found. Press 'n' to create one.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    let header = Row::new(vec![
        Cell::from(" ").style(Style::default().fg(Color::DarkGray)),
        Cell::from("Status").style(Style::default().fg(Color::White).bold()),
        Cell::from("Title").style(Style::default().fg(Color::White).bold()),
        Cell::from("Scheduled").style(Style::default().fg(Color::Yellow).bold()),
        Cell::from("Duration").style(Style::default().fg(Color::White).bold()),
        Cell::from("I/E").style(Style::default().fg(Color::Magenta).bold()),
        Cell::from("Category").style(Style::default().fg(Color::Blue).bold()),
        Cell::from("ID").style(Style::default().fg(Color::DarkGray).bold()),
    ])
    .height(1);

    let rows: Vec<Row> = commitments
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let selected = i == app.commitment_list_index;
            let (icon, status_str, color) = match c.status {
                CommitmentStatus::Completed => ("✓", "Done", Color::Green),
                CommitmentStatus::Missed => ("✗", "Missed", Color::Red),
                CommitmentStatus::Active => ("▶", "Active", Color::Yellow),
                CommitmentStatus::Planned => ("○", "Planned", Color::White),
                CommitmentStatus::Excused => ("⊘", "Excused", Color::DarkGray),
            };

            let time = c
                .scheduled_start
                .with_timezone(&Local)
                .format("%m-%d %H:%M")
                .to_string();

            let row_style = if selected {
                Style::default().bg(Color::DarkGray).fg(color)
            } else {
                Style::default().fg(color)
            };

            let pointer = if selected { "▸" } else { " " };

            Row::new(vec![
                Cell::from(pointer),
                Cell::from(format!("{} {}", icon, status_str)),
                Cell::from(c.title.clone()),
                Cell::from(time),
                Cell::from(fmt_duration(c.scheduled_duration_mins)),
                Cell::from(format!("{}/{}", c.importance, c.expected_effort)),
                Cell::from(c.category.clone()),
                Cell::from(c.id.to_string()[..8].to_string()),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(2),
        Constraint::Length(10),
        Constraint::Min(20),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(5),
        Constraint::Length(12),
        Constraint::Length(9),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(table, inner);
}
