use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};

use forgex_core::CommitmentStatus;
use crate::app::AppContext;
use crate::tui::widgets::fmt_duration;

pub fn render(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let block = Block::default()
        .title(" Recovery Center ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let active_penalties = ctx.db.get_active_penalties().unwrap_or_default();
    let commitments = ctx.db.list_commitments().unwrap_or_default();

    let total_remaining: u32 = active_penalties
        .iter()
        .map(|p| p.remaining_penalty_mins)
        .sum();
    let max_restricted: u32 = active_penalties
        .iter()
        .map(|p| p.restricted_days)
        .max()
        .unwrap_or(0);

    let save_days = ctx.db.get_available_save_days().unwrap_or_default();

    let chunks = Layout::vertical([
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Min(3),
    ])
    .split(inner);

    // Status
    {
        let text = if total_remaining == 0 && max_restricted == 0 {
            vec![
                Line::from(Span::styled(
                    "  ✓ No active penalties or restrictions!",
                    Style::default().fg(Color::Green).bold(),
                )),
                Line::from(Span::styled(
                    "  You have full freedom. Keep building your streak to earn Save Days.",
                    Style::default().fg(Color::Green),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Save Days available: "),
                    Span::styled(
                        save_days.len().to_string(),
                        Style::default().fg(Color::Green).bold(),
                    ),
                ]),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::raw("  Active Restriction: "),
                    Span::styled(
                        fmt_duration(total_remaining),
                        Style::default().fg(Color::Red).bold(),
                    ),
                    Span::raw(" remaining across "),
                    Span::styled(
                        max_restricted.to_string(),
                        Style::default().fg(Color::Red).bold(),
                    ),
                    Span::raw(" restricted day(s)"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  Save Days available: "),
                    Span::styled(
                        save_days.len().to_string(),
                        Style::default()
                            .fg(if save_days.is_empty() {
                                Color::Yellow
                            } else {
                                Color::Green
                            })
                            .bold(),
                    ),
                ]),
            ]
        };
        let p = Paragraph::new(text);
        frame.render_widget(p, chunks[0]);
    }

    // How to recover
    {
        let guide = vec![
            Line::from(Span::styled(
                "  HOW TO RECOVER:",
                Style::default().fg(Color::Yellow).bold(),
            )),
            Line::from(Span::styled(
                "  • Effort 1 task → Recovers 15m penalty + 5m entertainment",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "  • Effort 3 task → Recovers 45m penalty + 15m entertainment",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "  • Effort 5 task → Recovers 75m penalty + 25m entertainment",
                Style::default().fg(Color::White),
            )),
            Line::from(Span::styled(
                "  • Clearing all minutes reduces restricted days by 1",
                Style::default().fg(Color::DarkGray),
            )),
        ];
        let p = Paragraph::new(guide);
        frame.render_widget(p, chunks[1]);
    }

    // Available tasks
    {
        let planned: Vec<_> = commitments
            .iter()
            .filter(|c| {
                c.status == CommitmentStatus::Planned || c.status == CommitmentStatus::Active
            })
            .collect();

        if planned.is_empty() {
            let p = Paragraph::new("  No upcoming tasks. Go to Commitments [2] to create one.")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(p, chunks[2]);
        } else {
            let items: Vec<ListItem> = planned
                .iter()
                .map(|t| {
                    let potential = t.expected_effort as u32 * 15;
                    let text = format!(
                        "  [{}] {} (Effort {}/5) → Earns {} recovery",
                        &t.id.to_string()[..8],
                        t.title,
                        t.expected_effort,
                        fmt_duration(potential)
                    );
                    ListItem::new(text).style(Style::default().fg(Color::Cyan))
                })
                .collect();

            let header = Paragraph::new("  Available tasks to complete for recovery:")
                .style(Style::default().fg(Color::Cyan).bold());
            
            let header_chunks = Layout::vertical([
                Constraint::Length(1),
                Constraint::Min(1),
            ]).split(chunks[2]);

            frame.render_widget(header, header_chunks[0]);
            let list = List::new(items);
            frame.render_widget(list, header_chunks[1]);
        }
    }
}
