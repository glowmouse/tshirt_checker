use crate::loaded_image::*;
use crate::report_templates::*;

#[derive(PartialEq, Copy, Clone)]
pub enum Icon {
    Pass,
    Warn,
    Fail,
    Loading,
    Tool,
    Import,
    FixPT,
}

pub struct IconStorage {
    pass: LoadedImage,
    warn: LoadedImage,
    fail: LoadedImage,
    loading: LoadedImage,
    tool: LoadedImage,
    import: LoadedImage,
    partial_transparency_fix: LoadedImage,
}

impl IconStorage {
    pub fn new(ctx: &egui::Context) -> Self {
        let pass: LoadedImage =
            load_image_from_trusted_source(include_bytes!("pass.png"), "pass", ctx);
        let warn: LoadedImage =
            load_image_from_trusted_source(include_bytes!("warn.png"), "warn", ctx);
        let fail: LoadedImage =
            load_image_from_trusted_source(include_bytes!("fail.png"), "fail", ctx);
        let tool: LoadedImage =
            load_image_from_trusted_source(include_bytes!("tool.png"), "tool", ctx);
        let loading: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner.png"), "loading", ctx);
        let import: LoadedImage =
            load_image_from_trusted_source(include_bytes!("import_80x80.png"), "import", ctx);
        let partial_transparency_fix: LoadedImage =
            load_image_from_trusted_source(include_bytes!("partialt_80x80.png"), "partialt", ctx);

        Self {
            pass,
            warn,
            fail,
            loading,
            tool,
            import,
            partial_transparency_fix,
        }
    }

    pub fn get_loaded_image(&self, icon: Icon) -> &LoadedImage {
        match icon {
            Icon::Pass => &self.pass,
            Icon::Warn => &self.warn,
            Icon::Fail => &self.fail,
            Icon::Loading => &self.loading,
            Icon::Tool => &self.tool,
            Icon::Import => &self.import,
            Icon::FixPT => &self.partial_transparency_fix,
        }
    }
    pub fn texture_handle(&self, icon: Icon) -> &egui::TextureHandle {
        self.get_loaded_image(icon).texture_handle()
    }

    pub fn status_icon(&self, status: ReportStatus) -> egui::Image<'_> {
        egui::Image::from_texture(match status {
            ReportStatus::Unknown => self.texture_handle(Icon::Loading),
            ReportStatus::Fail => self.texture_handle(Icon::Fail),
            ReportStatus::Warn => self.texture_handle(Icon::Warn),
            ReportStatus::Pass => self.texture_handle(Icon::Pass),
        })
        .max_width(25.0)
    }

    fn image(&self, icon: Icon, width: f32) -> egui::Image<'_> {
        egui::Image::from_texture(self.texture_handle(icon)).max_width(width)
    }

    pub fn button(&self, icon: Icon, width: f32) -> egui::widgets::ImageButton<'_> {
        egui::widgets::ImageButton::new(self.image(icon, width).bg_fill(egui::Color32::WHITE))
    }
}
