use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Clear};

/// Centered popup area helper
#[allow(dead_code)]
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

/// Fixed size centered popup
pub fn centered_rect_fixed(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}

/// Renders a styled high-contrast footer shortcut bar
pub fn render_footer_shortcuts(frame: &mut Frame, area: Rect, shortcuts: &[(&str, &str)]) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in shortcuts.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default().fg(Color::Black).bg(Color::Yellow).bold(),
        ));
        spans.push(Span::styled(
            format!(" {} ", desc),
            Style::default().fg(Color::White).bold(),
        ));
    }

    let line = Line::from(spans);
    let footer = Paragraph::new(line)
        .alignment(Alignment::Center)
        .style(Style::default().bg(Color::Reset));
    frame.render_widget(footer, area);
}

/// Fallback text footer renderer
#[allow(dead_code)]
pub fn render_footer(frame: &mut Frame, area: Rect, text: &str) {
    let footer = Paragraph::new(text)
        .style(Style::default().fg(Color::White).bold())
        .alignment(Alignment::Center);
    frame.render_widget(footer, area);
}

/// A simple progress bar as a string
pub fn progress_str(val: u32, max: u32, width: usize) -> String {
    if max == 0 {
        return "─".repeat(width);
    }
    let filled = ((val as f32 / max as f32) * width as f32).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

/// Format minutes as human-readable duration
pub fn fmt_duration(mins: u32) -> String {
    if mins < 60 {
        format!("{}m", mins)
    } else {
        let h = mins / 60;
        let m = mins % 60;
        if m == 0 {
            format!("{}h", h)
        } else {
            format!("{}h {}m", h, m)
        }
    }
}

/// Renders a confirmation dialog. Returns the Rect used so the caller can
/// render the dialog on top of other content.
pub fn render_confirm_dialog(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    confirm_selected: bool,
) {
    let dialog_area = centered_rect_fixed(50, 9, area);
    frame.render_widget(Clear, dialog_area);

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let inner = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let chunks = Layout::vertical([
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .split(inner);

    let msg = Paragraph::new(message)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White));
    frame.render_widget(msg, chunks[0]);

    let confirm_style = if confirm_selected {
        Style::default().fg(Color::Black).bg(Color::Green).bold()
    } else {
        Style::default().fg(Color::Green)
    };
    let cancel_style = if !confirm_selected {
        Style::default().fg(Color::Black).bg(Color::Red).bold()
    } else {
        Style::default().fg(Color::Red)
    };

    let buttons = Line::from(vec![
        Span::styled("  [ Confirm ]  ", confirm_style),
        Span::raw("   "),
        Span::styled("  [ Cancel ]  ", cancel_style),
    ]);
    let btn_para = Paragraph::new(buttons).alignment(Alignment::Center);
    frame.render_widget(btn_para, chunks[2]);
}
