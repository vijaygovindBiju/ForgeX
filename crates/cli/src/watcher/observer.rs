use std::process::Command;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveWindow {
    pub app_class: String,
    pub title: String,
    pub pid: Option<u32>,
}

pub struct WindowObserver;

impl WindowObserver {
    pub fn get_active_window() -> Option<ActiveWindow> {
        // 1. Try Hyprland via hyprctl
        if let Some(win) = Self::get_via_hyprctl() {
            return Some(win);
        }

        // 2. Try xprop fallback (for X11 or XWayland)
        if let Some(win) = Self::get_via_xprop() {
            return Some(win);
        }

        None
    }

    fn get_via_hyprctl() -> Option<ActiveWindow> {
        let output = Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json_str = String::from_utf8(output.stdout).ok()?;
        let parsed: Value = serde_json::from_str(&json_str).ok()?;

        let class = parsed["class"].as_str().unwrap_or("").trim().to_string();
        let title = parsed["title"].as_str().unwrap_or("").trim().to_string();
        let pid = parsed["pid"].as_u64().map(|p| p as u32);

        if class.is_empty() && title.is_empty() {
            return None;
        }

        Some(ActiveWindow {
            app_class: if class.is_empty() { "Unknown".into() } else { class },
            title,
            pid,
        })
    }

    fn get_via_xprop() -> Option<ActiveWindow> {
        let root_output = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output()
            .ok()?;

        if !root_output.status.success() {
            return None;
        }

        let root_str = String::from_utf8(root_output.stdout).ok()?;
        let win_id = root_str.split('#').nth(1)?.trim();
        if win_id.is_empty() || win_id == "0x0" {
            return None;
        }

        let win_output = Command::new("xprop")
            .args(["-id", win_id, "WM_CLASS", "_NET_WM_NAME"])
            .output()
            .ok()?;

        let win_str = String::from_utf8(win_output.stdout).ok()?;

        let mut class = String::new();
        let mut title = String::new();

        for line in win_str.lines() {
            if line.starts_with("WM_CLASS") {
                if let Some(val) = line.split('=').nth(1) {
                    class = val.replace('"', "").trim().to_string();
                }
            } else if line.starts_with("_NET_WM_NAME") {
                if let Some(val) = line.split('=').nth(1) {
                    title = val.replace('"', "").trim().to_string();
                }
            }
        }

        if class.is_empty() && title.is_empty() {
            return None;
        }

        Some(ActiveWindow {
            app_class: class,
            title,
            pid: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_observer_execution() {
        // Observer should execute without panicking on the host system
        let _win = WindowObserver::get_active_window();
    }
}
