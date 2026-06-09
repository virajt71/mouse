use super::MouserApp;
use eframe::egui;

impl MouserApp {
    /// Upload the pre-decoded main mouse image to the GPU on first call; returns
    /// None if the background thread hasn't finished decoding yet.
    pub fn get_or_load_mouse_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.mouse_texture.is_none() {
            if let Some(image) = self.preloaded_mouse_image.take() {
                self.mouse_texture = Some(ctx.load_texture(
                    "mx_master_mouse_main",
                    image,
                    Default::default(),
                ));
            }
        }
        self.mouse_texture.clone()
    }

    /// Upload the pre-decoded customization mouse image to the GPU on first call;
    /// returns None if the background thread hasn't finished decoding yet.
    pub fn get_or_load_customization_mouse_texture(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<egui::TextureHandle> {
        if self.customization_mouse_texture.is_none() {
            if let Some(image) = self.preloaded_customization_mouse_image.take() {
                self.customization_mouse_texture = Some(ctx.load_texture(
                    "mx_master_mouse_customization",
                    image,
                    Default::default(),
                ));
            }
        }
        self.customization_mouse_texture.clone()
    }
}
