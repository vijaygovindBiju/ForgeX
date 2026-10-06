use colored::Colorize;

pub fn header(title: &str) -> String {
    format!(
        "\n{}\n{}",
        title.bold().cyan(),
        "─".repeat(40).dimmed()
    )
}

pub fn success_icon() -> String {
    "✓".green().bold().to_string()
}

pub fn missed_icon() -> String {
    "✗".red().bold().to_string()
}

pub fn active_icon() -> String {
    "▶".yellow().bold().to_string()
}

pub fn planned_icon() -> String {
    "○".dimmed().to_string()
}

pub fn excused_icon() -> String {
    "⊘".dimmed().to_string()
}

pub fn progress_bar(val: u32, max: u32, width: usize) -> String {
    if max == 0 {
        return format!("[{}]", "─".repeat(width).dimmed());
    }
    let filled_len = ((val as f32 / max as f32) * (width as f32)).round() as usize;
    let filled_len = filled_len.min(width);
    let empty_len = width - filled_len;

    format!(
        "[{}{}]",
        "█".repeat(filled_len).green(),
        "░".repeat(empty_len).dimmed()
    )
}

pub fn format_duration(mins: u32) -> String {
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
