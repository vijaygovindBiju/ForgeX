use chrono::{Local, Utc};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};

use forgex_core::CommitmentStatus;
use forgex_scoring::CharacterCalculator;
use crate::app::AppContext;
use crate::tui::widgets::{fmt_duration, progress_str};
use crate::tui::app::get_today_commitments;

pub fn render(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let chunks = Layout::vertical([
        Constraint::Length(7),  // Today's commitments summary
        Constraint::Length(6),  // Behavior + Penalty
        Constraint::Length(5),  // Save Days + Streak + Character Level
        Constraint::Min(0),     // Recovery / Tips
    ])
    .split(area);

    render_today_commitments(frame, chunks[0], ctx);
    render_behavior_penalty(frame, chunks[1], ctx);
    render_save_days_streak(frame, chunks[2], ctx);
    render_tips(frame, chunks[3], ctx);
}

fn render_today_commitments(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let today = get_today_commitments(ctx);

    let block = Block::default()
        .title(" Today's Commitments ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if today.is_empty() {
        let p = Paragraph::new("  No commitments scheduled for today. Press 'n' to create one.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(p, inner);
        return;
    }

    let items: Vec<ListItem> = today
        .iter()
        .map(|c| {
            let (icon, color) = match c.status {
                CommitmentStatus::Completed => ("✓", Color::Green),
                CommitmentStatus::Missed => ("✗", Color::Red),
                CommitmentStatus::Active => ("▶", Color::Yellow),
                CommitmentStatus::Planned => ("○", Color::White),
                CommitmentStatus::Excused => ("⊘", Color::DarkGray),
            };
            let time = c
                .scheduled_start
                .with_timezone(&Local)
                .format("%H:%M")
                .to_string();
            let dur = fmt_duration(c.scheduled_duration_mins);
            let text = format!(
                "{} {:>5}  {:<30}  {:>6}  [I:{} E:{}]",
                icon, time, c.title, dur, c.importance, c.expected_effort
            );
            ListItem::new(text).style(Style::default().fg(color))
        })
        .collect();

    let list = List::new(items);
    frame.render_widget(list, inner);
}

fn render_behavior_penalty(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let halves = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    // Left: behavior summary
    {
        let block = Block::default()
            .title(" Behavior ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let inner = block.inner(halves[0]);
        frame.render_widget(block, halves[0]);

        let all = ctx.db.list_commitments().unwrap_or_default();
        let max_skips = all.iter().map(|c| c.consecutive_skips).max().unwrap_or(0);

        let today_local = Local::now().date_naive();
        let today_ent: u32 = ctx
            .db
            .list_activities()
            .unwrap_or_default()
            .iter()
            .filter(|a| {
                a.started_at.with_timezone(&Local).date_naive() == today_local
                    && a.category.is_entertainment()
            })
            .map(|a| a.duration_mins)
            .sum();

        let penalties = ctx.db.list_penalties().unwrap_or_default();
        let avoidance = penalties.first().map(|p| p.avoidance_score).unwrap_or(0);

        let skip_color = if max_skips > 0 { Color::Red } else { Color::Green };
        let ent_color = if today_ent > 60 { Color::Red } else if today_ent > 30 { Color::Yellow } else { Color::Green };
        let avoid_color = if avoidance > 50 { Color::Red } else if avoidance > 0 { Color::Yellow } else { Color::Green };

        let text = vec![
            Line::from(vec![
                Span::raw("  Skip streak:    "),
                Span::styled(max_skips.to_string(), Style::default().fg(skip_color).bold()),
            ]),
            Line::from(vec![
                Span::raw("  Entertainment:  "),
                Span::styled(fmt_duration(today_ent), Style::default().fg(ent_color).bold()),
            ]),
            Line::from(vec![
                Span::raw("  Avoidance:      "),
                Span::styled(avoidance.to_string(), Style::default().fg(avoid_color).bold()),
            ]),
        ];

        let p = Paragraph::new(text);
        frame.render_widget(p, inner);
    }

    // Right: penalty summary
    {
        let active_penalties = ctx.db.get_active_penalties().unwrap_or_default();
        let total_remaining: u32 = active_penalties.iter().map(|p| p.remaining_penalty_mins).sum();
        let max_restricted: u32 = active_penalties.iter().map(|p| p.restricted_days).max().unwrap_or(0);

        let block = Block::default()
            .title(" Penalty ")
            .borders(Borders::ALL)
            .border_style(if total_remaining > 0 {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            });
        let inner = block.inner(halves[1]);
        frame.render_widget(block, halves[1]);

        let text = if total_remaining == 0 && max_restricted == 0 {
            vec![
                Line::from(Span::styled("  ✓ No active penalties", Style::default().fg(Color::Green).bold())),
                Line::from(Span::styled("  Full freedom!", Style::default().fg(Color::Green))),
            ]
        } else {
            vec![
                Line::from(vec![
                    Span::raw("  Remaining:      "),
                    Span::styled(fmt_duration(total_remaining), Style::default().fg(Color::Red).bold()),
                ]),
                Line::from(vec![
                    Span::raw("  Restricted days: "),
                    Span::styled(max_restricted.to_string(), Style::default().fg(Color::Red).bold()),
                ]),
                Line::from(Span::styled("  Complete tasks to recover!", Style::default().fg(Color::DarkGray).italic())),
            ]
        };

        let p = Paragraph::new(text);
        frame.render_widget(p, inner);
    }
}

fn render_save_days_streak(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let block = Block::default()
        .title(" Save Days & Consistency ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let available = ctx.db.get_available_save_days().unwrap_or_default().len();
    let all = ctx.db.list_commitments().unwrap_or_default();

    let now = Utc::now();
    let completed_week = all
        .iter()
        .filter(|c| {
            let days_ago = (now - c.scheduled_start).num_days();
            (0..=7).contains(&days_ago) && c.status == CommitmentStatus::Completed
        })
        .count();
    let scheduled_week = all
        .iter()
        .filter(|c| {
            let days_ago = (now - c.scheduled_start).num_days();
            (0..=7).contains(&days_ago)
        })
        .count();

    let ratio = if scheduled_week > 0 {
        (completed_week as f32 / scheduled_week as f32).min(1.0)
    } else {
        1.0
    };
    let consistency_days = (ratio * 7.0).round() as u32;

    // Character level
    let penalties = ctx.db.list_penalties().unwrap_or_default();
    let profile = CharacterCalculator::calculate(&all, &penalties);

    let save_color = if available == 0 { Color::Yellow } else { Color::Green };

    let text = vec![
        Line::from(vec![
            Span::raw("  Save Days: "),
            Span::styled(available.to_string(), Style::default().fg(save_color).bold()),
            Span::styled(format!("/{}", ctx.config.save_days_max), Style::default().fg(Color::DarkGray)),
            Span::raw("     Weekly: "),
            Span::styled(format!("{}/7", consistency_days), Style::default().bold()),
            Span::raw(" "),
            Span::styled(progress_str(consistency_days, 7, 10), Style::default().fg(Color::Green)),
            Span::raw("     Level: "),
            Span::styled(profile.overall_level.to_string(), Style::default().fg(Color::Cyan).bold()),
        ]),
    ];

    let p = Paragraph::new(text);
    frame.render_widget(p, inner);
}

fn render_tips(frame: &mut Frame, area: Rect, ctx: &AppContext) {
    let block = Block::default()
        .title(" Quick Tips ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let overdue = ctx.check_overdue_commitments().unwrap_or_default();
    let active_penalties = ctx.db.get_active_penalties().unwrap_or_default();

    let mut tips: Vec<Line> = Vec::new();
    if !overdue.is_empty() {
        tips.push(Line::from(Span::styled(
            format!("  ⚠ {} commitment(s) overdue! Go to Commitments [2] to handle them.", overdue.len()),
            Style::default().fg(Color::Yellow).bold(),
        )));
    }
    if !active_penalties.is_empty() {
        tips.push(Line::from(Span::styled(
            "  ⚡ Complete tasks to reduce your penalty! See Recovery [4] for details.",
            Style::default().fg(Color::Yellow),
        )));
    }
    if tips.is_empty() {
        tips.push(Line::from(Span::styled(
            "  ✓ You're on track. Keep building your consistency streak!",
            Style::default().fg(Color::Green).bold(),
        )));
    }
    tips.push(Line::from(vec![
        Span::raw("  💡 Shortcuts: "),
        Span::styled(" [n] ", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
        Span::styled(" New Task   ", Style::default().fg(Color::White).bold()),
        Span::styled(" [l] ", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
        Span::styled(" Log Activity   ", Style::default().fg(Color::White).bold()),
        Span::styled(" [?] ", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
        Span::styled(" Full User & Tracking Guide", Style::default().fg(Color::Cyan).bold()),
    ]));

    let p = Paragraph::new(tips);
    frame.render_widget(p, inner);
}
