/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    count: u32,
    #[serde(skip)] // This how you opt-out of serialization of a field
    dialog_handle: Option<async_std::task::JoinHandle<Vec<u8>>>,
    #[serde(skip)] // This how you opt-out of serialization of a field
    image_data: [u8; 512*512 ],
}

// Sketchy global so I can test stuff out while I struggle with the
// file dialog box code.
static mut HELLO: String = String::new();

fn load_image_from_memory(image_data: &[u8]) -> Result<egui::ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(egui::ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "AHello World 3!".to_owned(),
            value: 2.7,
            count: 0,
            dialog_handle: None,
            image_data: [0; 512*512 ],
        }
    }
}

impl TemplateApp {
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

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                ui.menu_button("File", |ui| {
                    if !is_web {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                    if ui.button("Load").clicked() {
                        async_std::task::block_on(
                            async {
                                let file = rfd::AsyncFileDialog::new().pick_file().await;
                                let data : Vec<u8> = file.unwrap().read().await;
                                unsafe {
                                    HELLO = data.len().to_string();
                                }
                                return data;
                            }
                        );
                        ui.close_menu();
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
ui.heading("eframe template");
ui.heading("eframe template2");
ui.heading("eframe template2");

        let _image_result = load_image_from_memory(&self.image_data);

            egui::ScrollArea::both().show(ui, |ui| {
                ui.add(
//                    egui::Image::new("https://en.wikipedia.org/wiki/PNG#/media/File:PNG_transparency_demonstration_1.png").rounding(10.0),
                    egui::Image::new("https://picsum.photos/seed/1.759706314/1024").rounding(10.0),
                );
            });

//            egui::ScrollArea::both().show(ui, |ui| {
//                ui.add(
//                    egui::Image::new("https://picsum.photos/seed/1.759706314/1024").rounding(10.0),
//                );
//            });

            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    unsafe {
                    ui.label("Bytes in file: ");
                    ui.label(&HELLO);
                    }
                });

                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        if self.dialog_handle.is_some() {
            let _handle = self.dialog_handle.as_ref().unwrap();
            self.label = "unwrapped handle ".to_string() + &(self.count.to_string());
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
