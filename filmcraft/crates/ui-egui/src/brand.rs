//! The FilmCraft app icon (`assets/app-icon/`, MIT OR Apache-2.0) as the logo of the About tab and
//! the start screen, decoded once and kept as a texture.

const APP_ICON: &[u8] = include_bytes!("../../../assets/app-icon/hicolor/128x128/apps/ai.storyteller.filmcraft.png");

/// The app icon's texture (square).
pub fn logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let id = egui::Id::new("app-icon-logo");
    if let Some(t) = ctx.data(|d| d.get_temp::<egui::TextureHandle>(id)) {
        return Some(t);
    }
    let img = image::load_from_memory_with_format(APP_ICON, image::ImageFormat::Png).ok()?.to_rgba8();
    let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
    let tex = ctx.load_texture("app-icon-logo", color, egui::TextureOptions::LINEAR);
    ctx.data_mut(|d| d.insert_temp(id, tex.clone()));
    Some(tex)
}

/// Draw the logo `height` points tall with its left edge at `left_center` (vertically centred).
/// Returns the drawn rect.
pub fn paint_logo(ui: &egui::Ui, left_center: egui::Pos2, height: f32) -> Option<egui::Rect> {
    let tex = logo(ui.ctx())?;
    let r = egui::Rect::from_min_size(egui::pos2(left_center.x, left_center.y - height / 2.0), egui::vec2(height, height));
    ui.painter().image(tex.id(), r, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);
    Some(r)
}
