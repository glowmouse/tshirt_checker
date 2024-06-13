use rand::Rng;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp<'a> {
    // Example stuff:
    _image_data: [u8; 262 * 304 * 4],
    rng: rand::rngs::ThreadRng,
    test_out: String,
    t_shirt_2: egui::Image<'a>,
    t_shirt: std::option::Option<egui::load::SizedTexture>
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

impl Default for TShirtCheckerApp<'_> {
    fn default() -> Self {
        Self {
            _image_data: [0; 262 * 304 * 4],
            rng: rand::thread_rng(),
            test_out: "".to_string(),
            t_shirt_2: egui::Image::new(egui::include_image!("blue_tshirt.png")) ,
            t_shirt: None
        }
    }
}

impl TShirtCheckerApp<'_> {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        //if let Some(storage) = cc.storage {
        //    return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //}

        let defaults : Self = Default::default();
                
        let test = egui::include_image!("blue_tshirt.png");
        let mut _nbytes: usize = 0;
        match test.clone() {
            egui::ImageSource::Uri(_a) => {
            },
            egui::ImageSource::Texture(_a) => {
            },
            egui::ImageSource::Bytes{uri : _, bytes } => {
                _nbytes = bytes.len();
            }
        }


        defaults                                                 
    }
}

fn mtext(text: &str) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

impl eframe::App for TShirtCheckerApp<'_> {

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        if Option::is_none(&self.t_shirt ) {
            let load_result = self.t_shirt_2.load_for_size( ctx, egui::Vec2::new(400.0,600.0) );
            if Result::is_ok(&load_result) {
                let texture_poll = load_result.unwrap();
                let osize = texture_poll.size();
                let oid = texture_poll.texture_id();
                if Option::is_some( &osize ) {
                    let size = osize.unwrap();
                    self.test_out = format!("size {} {} ", size[0], size[1] );
                }
                else {
                    self.test_out.push_str("no size ");
                }
                if Option::is_some( &oid ) {
                    self.test_out.push_str("Has ID");
                }
                else {
                    self.test_out.push_str("No ID ");
                }
                if Option::is_some( &osize ) && Option::is_some( &oid ) {
                    self.t_shirt = Some(egui::load::SizedTexture::new(oid.unwrap(), osize.unwrap()));
                }
            }
            else {
                self.test_out = "load failed".to_string();
            }
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

        egui::CentralPanel::default().show(ctx, |ui| {
            let (mut _response, painter ) =ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::drag() );
            if Option::is_some(&self.t_shirt ) {
                let sized_texture = self.t_shirt.unwrap();
                painter.image( 
                    sized_texture.id,
                    egui::Rect::from_min_max(egui::Pos2::new(0.0, 100.0), egui::Pos2::new(262.0*2.0, 304.0*2.0+100.0)),
                    egui::Rect::from_min_max(egui::Pos2::new(0.0, 0.0), egui::Pos2::new(1.0, 1.0)),
                    egui::Color32::WHITE );
            }
            let pos1 = egui::Pos2::new( self.rng.gen_range(0..500) as f32, self.rng.gen_range(0..500) as f32 );
            let pos2 = egui::Pos2::new( 150.0, 150.0 );
            painter.circle_filled( pos1, 60.0, egui::Color32::from_rgb(0, 255, 0 ));
            painter.circle_filled( pos2, 30.0, egui::Color32::from_rgb(255, 0, 0 ));
            
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| unsafe {
                    ui.label("Bytes in file: ");
                    let copy = HELLO.clone();
                    ui.label(&copy);
                });
                ui.horizontal(|ui| {
                    ui.label("Test Output  : ");
                    ui.label( &self.test_out );
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
