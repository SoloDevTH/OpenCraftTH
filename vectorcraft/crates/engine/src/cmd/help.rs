//! Help → community and project links (the OpenCraft Studio website, this app's page and its GitHub
//! repository). The commands return the URL; the frontend opens it.

use serde_json::{Value, json};

use super::*;

/// The app's id on github.com/SoloDevTH/OpenCraftTH and GitHub.
pub const APP_ID: &str = "vectorcraft";
pub const WEBSITE_URL: &str = "https://github.com/SoloDevTH/OpenCraftTH";

/// This app's page on the website.
pub fn app_page_url() -> String {
    format!("{WEBSITE_URL}/tree/main/{APP_ID}")
}

/// This app's source repository.
pub fn github_url() -> String {
    format!("{WEBSITE_URL}/tree/main/{APP_ID}")
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(query "help.website", "OpenCraft Studio Website", ["Help"], None, "{} → {url}", always, |_, _| url(WEBSITE_URL.into())),
        cmd!(query "help.appPage", "VectorCraft on github.com/SoloDevTH/OpenCraftTH", ["Help"], None, "{} → {url} this app's page", always, |_, _| url(app_page_url())),
        cmd!(query "help.github", "VectorCraft on GitHub", ["Help"], None, "{} → {url} source code, issues and releases", always, |_, _| url(github_url())),
        cmd!(query "help.links", "Links", [], None, "{} → {website, appPage, github}", always, |_, _| {
            Ok(json!({ "website": WEBSITE_URL, "appPage": app_page_url(), "github": github_url() }))
        }),
    ]
}

fn url(u: String) -> Result<Value> {
    Ok(json!({ "url": u }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links() {
        let mut s = Session::new();
        assert!(s.execute("help.discord", &json!({})).is_err(), "no Discord link");
        assert_eq!(s.execute("help.appPage", &json!({})).unwrap()["url"], "https://github.com/SoloDevTH/OpenCraftTH/tree/main/vectorcraft");
        assert_eq!(s.execute("help.github", &json!({})).unwrap()["url"], "https://github.com/SoloDevTH/OpenCraftTH/tree/main/vectorcraft");
        assert_eq!(s.execute("help.links", &json!({})).unwrap()["website"], "https://github.com/SoloDevTH/OpenCraftTH");
    }
}
