use forgex_core::ActivityCategory;

pub struct ActivityClassifier;

impl ActivityClassifier {
    pub fn classify(app_class: &str, window_title: &str) -> ActivityCategory {
        let class_lower = app_class.to_lowercase();
        let title_lower = window_title.to_lowercase();

        // 1. High Entertainment checks
        if Self::is_high_entertainment(&class_lower, &title_lower) {
            return ActivityCategory::HighEntertainment;
        }

        // 2. Productive checks (takes precedence over medium entertainment, e.g. YouTube tutorial -> Productive)
        if Self::is_productive(&class_lower, &title_lower) {
            return ActivityCategory::Productive;
        }

        // 3. Medium Entertainment checks (general YouTube, Netflix, movies)
        if Self::is_medium_entertainment(&class_lower, &title_lower) {
            return ActivityCategory::MediumEntertainment;
        }

        // 4. Essential checks
        if Self::is_essential(&class_lower, &title_lower) {
            return ActivityCategory::Essential;
        }

        ActivityCategory::Unknown
    }

    fn is_high_entertainment(class: &str, title: &str) -> bool {
        // Short-form and infinite scroll keywords
        let high_keywords = [
            "shorts", "reels", "tiktok", "instagram.com/reels",
            "youtube.com/shorts", "twitter.com", "x.com", "twitch.tv",
        ];
        if high_keywords.iter().any(|k| title.contains(k)) {
            return true;
        }

        // Games / Game Launchers
        let game_classes = [
            "steam", "lutris", "heroic", "retroarch", "prism launcher",
            "minecraft", "dota2", "cs2", "valheim",
        ];
        if game_classes.iter().any(|g| class.contains(g)) {
            return true;
        }

        false
    }

    fn is_medium_entertainment(class: &str, title: &str) -> bool {
        // Media players & streaming
        let medium_classes = [
            "vlc", "mpv", "spotify", "audacious", "rhythmbox",
        ];
        if medium_classes.iter().any(|m| class.contains(m)) {
            return true;
        }

        // Long-form video & casual browsing
        let medium_keywords = [
            "youtube", "netflix", "crunchyroll", "anime",
            "reddit", "bilibili", "hulu", "disneyplus", "prime video",
        ];
        if medium_keywords.iter().any(|k| title.contains(k)) {
            return true;
        }

        false
    }

    fn is_essential(class: &str, title: &str) -> bool {
        let essential_classes = [
            "keepassxc", "bitwarden", "1password", "gcr-prompter",
            "polkit", "gnome-calculator", "kcalc",
        ];
        if essential_classes.iter().any(|e| class.contains(e)) {
            return true;
        }

        let essential_keywords = [
            "bank", "banking", "login", "authenticate", "portal",
        ];
        if essential_keywords.iter().any(|k| title.contains(k)) {
            return true;
        }

        false
    }

    fn is_productive(class: &str, title: &str) -> bool {
        // Code editors & IDEs
        let dev_classes = [
            "code", "vscodium", "cursor", "nvim", "vim", "emacs", "zed",
            "sublime_text", "idea", "clion", "pycharm", "rustrover",
            "alacritty", "kitty", "ghostty", "foot", "wezterm", "gnome-terminal",
            "tmux", "terminal", "obsidian", "logseq",
        ];
        if dev_classes.iter().any(|d| class.contains(d)) {
            return true;
        }

        // Documentation, research, GitHub
        let dev_keywords = [
            "github.com", "gitlab.com", "stackoverflow.com", "docs.rs",
            "crates.io", "doc.rust-lang.org", "developer.android.com",
            "arxiv.org", "documentation", "tutorial", "claude", "chatgpt",
        ];
        if dev_keywords.iter().any(|k| title.contains(k)) {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_classification() {
        assert_eq!(
            ActivityClassifier::classify("com.mitchellh.ghostty", "PirateOS: /ForgeX"),
            ActivityCategory::Productive
        );
        assert_eq!(
            ActivityClassifier::classify("code", "main.rs - ForgeX - Visual Studio Code"),
            ActivityCategory::Productive
        );
        assert_eq!(
            ActivityClassifier::classify("firefox", "YouTube Shorts - Funny Clips"),
            ActivityCategory::HighEntertainment
        );
        assert_eq!(
            ActivityClassifier::classify("firefox", "Rust Tutorial - Lifetimes - YouTube"),
            ActivityCategory::Productive
        );
        assert_eq!(
            ActivityClassifier::classify("firefox", "Movie Trailer - Netflix"),
            ActivityCategory::MediumEntertainment
        );
        assert_eq!(
            ActivityClassifier::classify("firefox", "std::sync::Arc - docs.rs"),
            ActivityCategory::Productive
        );
        assert_eq!(
            ActivityClassifier::classify("steam", "Steam"),
            ActivityCategory::HighEntertainment
        );
        assert_eq!(
            ActivityClassifier::classify("keepassxc", "Passwords"),
            ActivityCategory::Essential
        );
    }
}
