pub const ICON_LOAD_ANIMATION_IN_MILLIS: u64 = 200;

use crate::loaded_image::*;
use crate::report_templates::*;
use web_time::SystemTime;

#[derive(PartialEq, Copy, Clone)]
pub enum Icon {
    Pass,
    Warn,
    Fail,
    Tool,
    Import,
    FixPT,
}

pub struct IconStorage {
    pass: LoadedImage,
    warn: LoadedImage,
    fail: LoadedImage,
    loading: [LoadedImage; 12],
    tool: LoadedImage,
    import: LoadedImage,
    partial_transparency_fix: LoadedImage,
    icon_last_cycle: SystemTime,
    cycle: usize,
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

        // TODO, right way to do this in Rust.
        let loading_01: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_01.png"), "loading", ctx);
        let loading_02: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_02.png"), "loading", ctx);
        let loading_03: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_03.png"), "loading", ctx);
        let loading_04: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_04.png"), "loading", ctx);
        let loading_05: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_05.png"), "loading", ctx);
        let loading_06: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_06.png"), "loading", ctx);
        let loading_07: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_07.png"), "loading", ctx);
        let loading_08: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_08.png"), "loading", ctx);
        let loading_09: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_09.png"), "loading", ctx);
        let loading_10: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_10.png"), "loading", ctx);
        let loading_11: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_11.png"), "loading", ctx);
        let loading_12: LoadedImage =
            load_image_from_trusted_source(include_bytes!("spinner_12.png"), "loading", ctx);
        let import: LoadedImage =
            load_image_from_trusted_source(include_bytes!("import_80x80.png"), "import", ctx);
        let partial_transparency_fix: LoadedImage =
            load_image_from_trusted_source(include_bytes!("partialt_80x80.png"), "partialt", ctx);

        Self {
            pass,
            warn,
            fail,
            loading: [
                loading_01, loading_02, loading_03, loading_04, loading_05, loading_06, loading_07,
                loading_08, loading_09, loading_10, loading_11, loading_12,
            ],
            tool,
            import,
            partial_transparency_fix,
            icon_last_cycle: SystemTime::now(),
            cycle: 0,
        }
    }

    pub fn get_loaded_image(&self, icon: Icon) -> &LoadedImage {
        match icon {
            Icon::Pass => &self.pass,
            Icon::Warn => &self.warn,
            Icon::Fail => &self.fail,
            Icon::Tool => &self.tool,
            Icon::Import => &self.import,
            Icon::FixPT => &self.partial_transparency_fix,
        }
    }
    pub fn texture_handle(&self, icon: Icon) -> &egui::TextureHandle {
        self.get_loaded_image(icon).texture_handle()
    }

    pub fn load_animation(&self) -> &egui::TextureHandle {
        self.loading[self.cycle % 12].texture_handle()
    }

    pub fn status_icon(&self, status: ReportStatus) -> egui::Image<'_> {
        egui::Image::from_texture(match status {
            ReportStatus::Unknown => self.load_animation(),
            ReportStatus::Fail => self.texture_handle(Icon::Fail),
            ReportStatus::Warn => self.texture_handle(Icon::Warn),
            ReportStatus::Pass => self.texture_handle(Icon::Pass),
        })
    }

    fn image(&self, icon: Icon, width: f32) -> egui::Image<'_> {
        egui::Image::from_texture(self.texture_handle(icon)).max_width(width)
    }

    pub fn button(&self, icon: Icon, width: f32) -> egui::widgets::ImageButton<'_> {
        egui::widgets::ImageButton::new(self.image(icon, width).bg_fill(egui::Color32::WHITE))
    }

    pub fn advance_cycle(&mut self) {
        let time_since_last_cycle: u64 = self
            .icon_last_cycle
            .elapsed()
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap();
        if time_since_last_cycle > ICON_LOAD_ANIMATION_IN_MILLIS - 20 {
            self.cycle += 1;
            self.icon_last_cycle = SystemTime::now()
        }
    }
}
