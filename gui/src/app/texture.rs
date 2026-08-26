use super::MouserApp;
use eframe::egui;

impl MouserApp {
    /// Upload the pre-decoded main mouse image to the GPU on first call; returns
    /// None if the background thread hasn't finished decoding yet.
    pub fn get_or_load_mouse_texture(
        &mut self,
        ctx: &egui::Context,
    ) -> Option<egui::TextureHandle> {
        if self.mouse_texture.is_none() {
            if let Some(image) = self.preloaded_mouse_image.take() {
                self.mouse_texture =
                    Some(ctx.load_texture("mx_master_mouse_main", image, Default::default()));
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

    /// Upload the device image to the GPU on first call and cache it.
    pub fn get_or_load_device_texture(
        &mut self,
        ctx: &egui::Context,
        layout_key: &str,
    ) -> Option<egui::TextureHandle> {
        if !self.device_textures.contains_key(layout_key) {
            let bytes: &[u8] = match layout_key {
                "mx_vertical" => include_bytes!("../../../assets/images/mx_vertical.png"),
                "mx_keys" | "mx_keys_s" | "mx_keys_mini" => {
                    include_bytes!("../../../assets/images/logitech-keyboards/mx_keys/keyboard.png")
                }
                "mx_master_3" | "mx_master_3s" | "mx_master_2s" | "mx_master" => {
                    include_bytes!("../../../assets/images/logitech-mice/mx_master_3/mouse.png")
                }
                "mx_anywhere_3s" | "mx_anywhere_3" | "mx_anywhere" => {
                    include_bytes!("../../../assets/images/logitech-mice/mx_anywhere_3s/mouse.png")
                }
                "mx_mechanical" | "mx_mechanical_mini" => {
                    include_bytes!(
                        "../../../assets/images/logitech-keyboards/mx_mechanical/front.png"
                    )
                }
                _ => include_bytes!("../../../assets/images/mouse.png"),
            };

            let img = image::load_from_memory(bytes).ok()?;
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            let color_img = egui::ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                &rgba.into_raw(),
            );
            let texture = ctx.load_texture(
                format!("device_tex_{}", layout_key),
                color_img,
                Default::default(),
            );
            self.device_textures.insert(layout_key.to_string(), texture);
        }
        self.device_textures.get(layout_key).cloned()
    }
}
