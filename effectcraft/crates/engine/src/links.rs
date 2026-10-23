//! Community and project links (shown in Help ▸, the About dialog, the start screen and the
//! header).

pub const APP_NAME: &str = "EffectCraft";
pub const APP_SLUG: &str = "effectcraft";
pub const WEBSITE: &str = "https://github.com/SoloDevTH/OpenCraftTH";
pub const APP_PAGE: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/effectcraft";
pub const GITHUB: &str = "https://github.com/SoloDevTH/OpenCraftTH/tree/main/effectcraft";
pub const ISSUES: &str = "https://github.com/SoloDevTH/OpenCraftTH/issues";

/// Sibling apps of the OpenCraft Studio family: (name, what it is, GitHub repo, app page).
pub const SIBLINGS: &[(&str, &str)] = &[
    ("PhotoCraft", "photocraft"),
    ("VectorCraft", "vectorcraft"),
    ("FilmCraft", "filmcraft"),
    ("LightCraft", "lightcraft"),
    ("PdfCraft", "pdfcraft"),
    ("DesignCraft", "designcraft"),
];

pub fn github_of(slug: &str) -> String {
    format!("https://github.com/SoloDevTH/OpenCraftTH/tree/main/{slug}")
}
pub fn page_of(slug: &str) -> String {
    format!("https://github.com/SoloDevTH/OpenCraftTH/tree/main/{slug}")
}
