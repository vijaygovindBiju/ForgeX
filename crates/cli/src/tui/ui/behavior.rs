use chrono::Local;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem};

use crate::app::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::widgets::fmt_duration;

pub fn render(frame: &mut Frame, area: Rect, app: &TuiApp, ctx: &AppContext) {
    let block = Block::default()
        .title(" Behavior & Activity Tracking — press 'l' to Log Activity ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan).bold());
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(7), // Tracking status & guide banner
        Constraint::Min(4),    // Activity list
    ])
    .split(inner);

    render_tracking_banner(frame, chunks[0], app, ctx);
    render_activity_list(frame, chunks[1], app, ctx);
}

fn render_tracking_banner(frame: &mut Frame, area: Rect, app: &TuiApp, ctx: &AppContext) {
    let sub_block = Block::default()
        .title(" Activity Tracking Status & Guide ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = sub_block.inner(area);
    frame.render_widget(sub_block, area);

    let today_local = Local::now().date_naive();
    let all_activities = ctx.db.list_activities().unwrap_or_default();
    let today_ent: u32 = all_activities
        .iter()
        .filter(|a| {
            a.started_at.with_timezone(&Local).date_naive() == today_local
                && a.category.is_entertainment()
        })
        .map(|a| a.duration_mins)
        .sum();

    let live_opt = app.live_info.read().ok().and_then(|guard| guard.clone());
    let auto_tracking_line = match live_opt {
        Some(live) => Line::from(vec![
            Span::styled("  👁️ Live Window:        ", Style::default().fg(Color::Green).bold()),
            Span::styled(format!("{:<16}", live.app_class), Style::default().fg(Color::Yellow).bold()),
            Span::raw(" │ "),
            Span::styled(
                format!("{:<32}", if live.title.len() > 32 { format!("{}...", &live.title[..29]) } else { live.title }),
                Style::default().fg(Color::White)
            ),
            Span::raw(" │ "),
            Span::styled(format!("{} ", live.category), Style::default().fg(Color::Magenta).bold()),
            Span::styled(format!("({}s active)", live.session_secs), Style::default().fg(Color::Cyan)),
        ]),
        None => Line::from(vec![
            Span::styled("  👁️ Live Watcher:       ", Style::default().fg(Color::Green).bold()),
            Span::raw("Monitoring active windows in background (Hyprland / Wayland / X11)."),
        ]),
    };

    let lines = vec![
        auto_tracking_line,
        Line::from(vec![
            Span::styled("  ✍️ Manual Logging:     ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("Press "),
            Span::styled("l", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
            Span::raw(" right here in the TUI to manually log activities (e.g. coding, study, games)."),
        ]),
        Line::from(vec![
            Span::styled("  📊 Today's Record:     ", Style::default().fg(Color::White).bold()),
            Span::raw(format!("Total activities: {}  │  Entertainment today: ", all_activities.len())),
            Span::styled(fmt_duration(today_ent), Style::default().fg(if today_ent > 60 { Color::Red } else { Color::Green }).bold()),
            Span::raw("  │  "),
            Span::styled("Press '?' for full guide", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_activity_list(frame: &mut Frame, area: Rect, app: &TuiApp, ctx: &AppContext) {
    let activities = ctx.db.list_activities().unwrap_or_default();

    if activities.is_empty() {
        let empty_msg = vec![
            Line::from(""),
            Line::from(Span::styled(
                "    No activities recorded yet!",
                Style::default().fg(Color::Yellow).bold(),
            )),
            Line::from(""),
            Line::from(Span::raw(
                "    Why is nothing tracked? Automatic monitoring requires the watcher to be running:",
            )),
            Line::from(vec![
                Span::raw("    1. Automatic: In another terminal, run: "),
                Span::styled("forgex watch", Style::default().fg(Color::Green).bold()),
            ]),
            Line::from(vec![
                Span::raw("    2. Manual: Press "),
                Span::styled("l", Style::default().fg(Color::Black).bg(Color::Yellow).bold()),
                Span::raw(" right now to log work or entertainment directly."),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "    (Entertainment logged during scheduled commitment windows affects avoidance penalties)",
                Style::default().fg(Color::DarkGray).italic(),
            )),
        ];
        frame.render_widget(Paragraph::new(empty_msg), area);
        return;
    }

    let start = app.activity_scroll as usize;
    let visible = area.height.saturating_sub(2) as usize;
    let end = (start + visible).min(activities.len());

    let header_line = Line::from(vec![
        Span::styled(format!("  {:<14}", "TIME"), Style::default().fg(Color::Yellow).bold()),
        Span::styled(format!("{:<22}", "APPLICATION"), Style::default().fg(Color::White).bold()),
        Span::styled(format!("{:<24}", "DETAIL / DOMAIN"), Style::default().fg(Color::Cyan).bold()),
        Span::styled(format!("{:<22}", "CATEGORY"), Style::default().fg(Color::Magenta).bold()),
        Span::styled(format!("{:<10}", "DURATION"), Style::default().fg(Color::White).bold()),
    ]);

    let mut items: Vec<ListItem> = Vec::new();
    items.push(ListItem::new(header_line));

    for a in &activities[start..end] {
        let time = a
            .started_at
            .with_timezone(&Local)
            .format("%m-%d %H:%M")
            .to_string();
        let detail = a
            .domain_or_detail
            .as_deref()
            .unwrap_or("-");
        let cat_color = match a.category {
            forgex_core::ActivityCategory::Productive => Color::Green,
            forgex_core::ActivityCategory::Essential => Color::Cyan,
            forgex_core::ActivityCategory::MediumEntertainment => Color::Yellow,
            forgex_core::ActivityCategory::HighEntertainment => Color::Red,
            forgex_core::ActivityCategory::Unknown => Color::DarkGray,
        };
        let text = Line::from(vec![
            Span::styled(format!("  {:<14}", time), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:<22}", a.application), Style::default().fg(Color::White).bold()),
            Span::styled(format!("{:<24}", detail), Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:<22}", a.category.to_string()), Style::default().fg(cat_color)),
            Span::styled(format!("{:<10}", fmt_duration(a.duration_mins)), Style::default().fg(Color::White)),
        ]);
        items.push(ListItem::new(text));
    }

    let list = List::new(items);
    frame.render_widget(list, area);
}
