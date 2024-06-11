use image::{RgbImage, Rgb};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp {
    image_buffer: image::ImageBuffer<Rgb<u8>, std::vec::Vec<u8>>,
    // Example stuff:
    _image_data: [u8; 262 * 304 * 4],
}

// Sketchy global so I can test stuff out while I struggle with the
// file dialog box code.
static mut HELLO: String = String::new();

//
// Copied from https://github.com/PolyMeilex/rfd/blob/master/examples/async.rs
//
// My current understanding (new to this) is that nothing executed in web 
// assembly can block the main thread...  and the thread mechanism used by
// web assembly won't return the thread's output.
//
use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
fn app_execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || async_std::task::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn app_execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

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
            image_buffer: RgbImage::new(32,32),
            _image_data: [0; 262 * 304 * 4],
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
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        for x in 15..=17 {
            for y in 8..24 {
                self.image_buffer.put_pixel(x, y, Rgb([255, 0, 0]));
                self.image_buffer.put_pixel(y, x, Rgb([255, 0, 0]));
            }
        }
        unsafe {
            HELLO = self.image_buffer.as_raw().len().to_string();
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            if ui.button(mtext("Load")).clicked() {
                // Execute in another thread
                app_execute(async {
                    unsafe {
                        HELLO = "here".to_string();
                    }
                    let file = rfd::AsyncFileDialog::new().pick_file().await;
                    unsafe {
                        HELLO = "there".to_string();
                    }
                    let data: Vec<u8> = file.unwrap().read().await;
                    unsafe {
                        HELLO = data.len().to_string();
                    }
                });
            }
        });

        //let my_image = image::load_from_memory(self.image_buffer.as_raw()).unwrap();
        //let my_image_buffer = my_image.to_rgba8();
        //let my_size = [my_image.width() as _, my_image.height() as _];
        //let my_pixels = my_image_buffer.as_flat_samples();
        //let _my_something = egui::ColorImage::from_rgba_unmultiplied(
        //    my_size,
        //    my_pixels.as_slice(),
        //);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                ui.image(egui::include_image!("blue_tshirt.png"));
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
