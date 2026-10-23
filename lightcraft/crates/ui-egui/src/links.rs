//! Community and project links (Help menu, About dialog).

/// The app's id in OpenCraft Studio URLs.
pub const APP: &str = "lightcraft";
/// The OpenCraft Studio website.
pub const WEBSITE: &str = "https://github.com/SoloDevTH/OpenCraftTH";
/// This app's page on the OpenCraft Studio website.
pub const APP_PAGE: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/lightcraft";
/// This app's source repository.
pub const GITHUB: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/lightcraft";

/// (UI command id, menu label, URL) for each link, in Help-menu order.
/// The user documentation (docs/ in the repository).
pub const HELP: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/lightcraft/docs";

pub const LINKS: &[(&str, &str, &str)] = &[
    ("app.help", "LightCraft Help", HELP),
    ("app.website", "LightCraft Website", APP_PAGE),
    ("app.github", "LightCraft on GitHub", GITHUB),
    ("app.artcraft", "OpenCraft Studio Website", WEBSITE),
    ("app.feedback", "Send Feedback…", FEEDBACK),
];

/// Where feedback and bug reports go.
pub const FEEDBACK: &str = "https://github.com/SoloDevTH/OpenCraftTH/issues/new";

/// The URL behind a link command id.
pub fn url_of(cmd: &str) -> Option<&'static str> {
    LINKS.iter().find(|(id, _, _)| *id == cmd).map(|(_, _, u)| *u)
}

/// Open `url` in the user's browser (through the host's `open_url` service).
pub fn open(app: &mut crate::LightcraftApp, url: &str) -> Result<serde_json::Value, String> {
    let open = app.services.open_url.as_mut().ok_or("can't open links here")?;
    open(url)?;
    Ok(serde_json::json!({ "url": url }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_https_and_name_this_app() {
        for (id, label, url) in LINKS {
            assert!(url.starts_with("https://"), "{id}");
            assert!(!label.is_empty());
            assert_eq!(url_of(id), Some(*url));
        }
        // OpenCraft Studio: the app's page and its code are its folder in the fork's repository
        assert!(APP_PAGE.ends_with(&format!("/tree/main/{APP}")));
        assert!(GITHUB.ends_with(&format!("/tree/main/{APP}")));
    }
}
