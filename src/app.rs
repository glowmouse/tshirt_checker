/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TShirtCheckerApp {
    // Example stuff:
    footer: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    count: u32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    dialog_handle: Option<async_std::task::JoinHandle<Vec<u8>>>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    image_data: [u8; 512 * 512],
}

// Sketchy global so I can test stuff out while I struggle with the
// file dialog box code.
static mut HELLO: String = String::new();

//fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
//    let image = image::load_from_memory(image_data)?;
//    let size = [image.width() as _, image.height() as _];
//    let image_buffer = image.to_rgba8();
//    let pixels = image_buffer.as_flat_samples();
//    Ok(egui::ColorImage::from_rgba_unmultiplied(
//        size,
//        pixels.as_slice(),
//    ))
//}

impl Default for TShirtCheckerApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            footer: "".to_owned(),
            value: 2.7,
            count: 0,
            dialog_handle: None,
            image_data: [0; 512 * 512],
        }
    }
}

impl TShirtCheckerApp {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        Default::default()
    }
}

fn mtext(text: &str) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

impl eframe::App for TShirtCheckerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if ui.button(mtext("Load")).clicked() {
                async_std::task::block_on(async {
                    let file = rfd::AsyncFileDialog::new().pick_file().await;
                    let data: Vec<u8> = file.unwrap().read().await;
                    unsafe {
                        HELLO = data.len().to_string();
                    }
                    data
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.image(egui::include_image!("blue_tshirt.png"))
            });

            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| unsafe {
                    ui.label("Bytes in file: ");
                    let copy = HELLO.clone();
                    ui.label(&copy);
                });

                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        if self.dialog_handle.is_some() {
            let _handle = self.dialog_handle.as_ref().unwrap();
            self.footer = "unwrapped handle ".to_string() + &(self.count.to_string());
            self.count += 1;
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
