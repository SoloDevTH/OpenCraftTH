//! OpenCraft Studio community and project links (Help menu, About, start screen, CLI, MCP).

use serde_json::{Value, json};

/// The app's slug on github.com/SoloDevTH/OpenCraftTH and GitHub.
pub const APP: &str = "designcraft";
/// The OpenCraft Studio website.
pub const WEBSITE: &str = "https://github.com/SoloDevTH/OpenCraftTH";
/// DesignCraft's page on the OpenCraft Studio website.
pub const APP_PAGE: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/designcraft";
/// DesignCraft's source repository.
pub const GITHUB: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/designcraft";
/// Where to report bugs and request features.
pub const ISSUES: &str = "https://github.com/SoloDevTH/OpenCraftTH/issues";

/// All links as JSON (the `app.links` command).
pub fn all() -> Value {
    json!({
        "website": WEBSITE,
        "appPage": APP_PAGE,
        "github": GITHUB,
        "issues": ISSUES,
    })
}

/// A link by key (`website`, `appPage`, `github`, `issues`).
pub fn get(key: &str) -> Option<&'static str> {
    Some(match key {
        "website" => WEBSITE,
        "appPage" => APP_PAGE,
        "github" => GITHUB,
        "issues" => ISSUES,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_follow_the_app_slug() {
        // OpenCraft Studio: the app's page and its code are its folder in the fork's repository
        assert!(APP_PAGE.ends_with(&format!("/tree/main/{APP}")));
        assert!(GITHUB.ends_with(&format!("/tree/main/{APP}")));
        assert!(ISSUES.starts_with(WEBSITE)); // one repository: its Issues are the suite's
        for k in ["website", "appPage", "github", "issues"] {
            assert!(get(k).is_some_and(|u| u.starts_with("https://")), "{k}");
            assert_eq!(all()[k], get(k).unwrap());
        }
    }
}
