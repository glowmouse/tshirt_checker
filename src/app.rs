//use rand::Rng;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp<'a> {
    // Example stuff:
    _rng: rand::rngs::ThreadRng,
    footer_debug_0: String,
    footer_debug_1: String,
    t_shirt_img_src: egui::Image<'a>,
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
fn _app_execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || async_std::task::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn _app_execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

impl Default for TShirtCheckerApp<'_> {
    fn default() -> Self {
        Self {
            _rng: rand::thread_rng(),
            footer_debug_0: "".to_string(),
            footer_debug_1: "".to_string(),
            t_shirt_img_src: egui::Image::new(egui::include_image!("blue_tshirt.png")) ,
            t_shirt: None
        }
    }
}

        /*
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            let size = ui.available_size_before_wrap();
            self.footer_debug = format!("{} {}", size[0], size[1] );

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
        */


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

    fn do_texture_loads(&mut self, ctx: &egui::Context ) {
        // If we don't have a Texture ID for the T-Shirt, push on the load.
        // The T-Shirt is compiled into the binary, so I don't expect to
        // see any weird load problems.
        // 
        if Option::is_none(&self.t_shirt ) {
            let load_result = self.t_shirt_img_src.load_for_size( ctx, egui::Vec2::new(1.0,1.0) );
            if Result::is_ok(&load_result) {
                let texture_poll = load_result.unwrap();
                let osize = texture_poll.size();
                let oid = texture_poll.texture_id();
                if Option::is_some( &osize ) && Option::is_some( &oid ) {
                    self.t_shirt = Some(egui::load::SizedTexture::new(oid.unwrap(), osize.unwrap()));
                }
            }
        }
    }

    fn do_bottom_panel(&self, ctx: &egui::Context ) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
                ui.horizontal(|ui| unsafe {
                    ui.label("Bytes in file: ");
                    let copy = HELLO.clone();
                    ui.label(&copy);
                });
                ui.horizontal(|ui| {
                    ui.label("footer_debug_0: ");
                    ui.label( &self.footer_debug_0 );
                });
                ui.horizontal(|ui| {
                    ui.label("footer_debug_1: ");
                    ui.label( &self.footer_debug_1 );
                });

                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
        });
    }

    fn do_central_panel(&mut self, ctx: &egui::Context ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_size = ui.available_size_before_wrap();

            let xscale = panel_size[0] / 262.0;
            let yscale = panel_size[1] / 304.0;
            let scale = f32::min( xscale, yscale );
            let uv0 = egui::Pos2{ x: 0.0, y: 0.0 };
            let uv1 = egui::Pos2{ x: 1.0, y: 1.0 };
            let img_size = egui::Pos2{ x: 262.0*scale, y: 304.0*scale };
            let s0  = egui::Pos2{ x: (panel_size[0] - img_size.x)/2.0, y: (panel_size[1] - img_size.y)/ 2.0 };
            let s1  = egui::Pos2{ x: s0.x + img_size.x, y: s0.y + img_size.y };

            //self.footer_debug_0 = format!("{} {}", panel_size[0], panel_size[1] );
            self.footer_debug_1 = format!("{} {}", s1[0], s1[1] );

            let (mut _response, painter ) =ui.allocate_painter(panel_size, egui::Sense::drag() );
            if Option::is_some(&self.t_shirt ) {
                let sized_texture = self.t_shirt.unwrap();
                painter.image( 
                    sized_texture.id,
                    egui::Rect::from_min_max(s0, s1 ),
                    egui::Rect::from_min_max(uv0, uv1 ),
                    egui::Color32::WHITE );
            }
        });
    }

    fn do_right_panel(&mut self, ctx: &egui::Context ) {
        egui::SidePanel::right("stuff")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
             ui.vertical_centered(|ui| {
                let panel_size = ui.available_size_before_wrap();
                self.footer_debug_0 = format!("{} {}", panel_size[0], panel_size[1] );
                ui.heading(mtext("T-Shirt Check"));
            })
        });
    }
}

fn mtext(text: &str) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

impl eframe::App for TShirtCheckerApp<'_> {


    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.do_texture_loads( ctx ); 
        self.do_bottom_panel( ctx );
        self.do_right_panel( ctx );
        self.do_central_panel( ctx );
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
