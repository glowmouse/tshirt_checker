
extern crate nalgebra as na;
use na::{Matrix3, matrix, dvector, vector, Vector3};

/// My image abstraction
pub struct LoadedImage {
    uncompressed_image:     egui::ColorImage,
    texture:                egui::TextureHandle,
}

impl LoadedImage {
    pub fn new( uncompressed_image_arg: egui::ColorImage, texture_arg: egui::TextureHandle ) -> Self {
      Self {
          uncompressed_image:   uncompressed_image_arg,
          texture:              texture_arg
      }
    }

    pub fn id(&self) -> egui::TextureId {
      self.texture.id()
    }

    pub fn size(&self) -> egui::Vec2 {
      self.texture.size_vec2()
    }

    pub fn pixels(&self) -> &Vec<egui::Color32> {
      &self.uncompressed_image.pixels
    }
}

fn load_image_from_trusted_source( bytes : &[u8],  name: impl Into<String>, ctx: &egui::Context ) -> LoadedImage 
{
    let uncompressed_image = egui_extras::image::load_image_bytes( bytes ).unwrap();
    let xsize = uncompressed_image.size[0];
    let ysize = uncompressed_image.size[1];
    let xsize_f = f32::from(i16::try_from(xsize).unwrap());
    let ysize_f = f32::from(i16::try_from(ysize).unwrap());
    let handle: egui::TextureHandle = ctx.load_texture( name, uncompressed_image.clone(), Default::default() );
    LoadedImage{ uncompressed_image: uncompressed_image, texture: handle }
 }

/*
  self.t_shirt_colorimage = Some( egui_extras::image::load_image_bytes( self.t_shirt_bytes ).unwrap() );


        if Option::is_none(&self.t_shirt_colorimage) {
            // Prototyping off https://docs.rs/egui/latest/egui/struct.Context.html#method.load_texture
            self.t_shirt_colorimage = Some( egui_extras::image::load_image_bytes( self.t_shirt_bytes ).unwrap() );
            let xsize = self.t_shirt_colorimage.as_ref().unwrap().size[0];
            let ysize = self.t_shirt_colorimage.as_ref().unwrap().size[1];
            let xsize_f = f32::from(i16::try_from(xsize).unwrap());
            let ysize_f = f32::from(i16::try_from(ysize).unwrap());

            let tshirt_size = handle.size_vec2();
            self.footer_debug_1 = format!("t_shirt size {} {} - {} {} - {} {}", xsize_f, ysize_f, handle.size_vec2().x, handle.size_vec2().y, tshirt_size.x, tshirt_size.y );
            self.t_shirt_2 = Some(handle);
        }
*/


/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp<'a> {
    footer_debug_0:         String,
    footer_debug_1:         String,
    t_shirt_bytes:          &'a[u8],
    t_shirt_colorimage:     std::option::Option<egui::ColorImage>,
    test_artwork_src:       egui::Image<'a>,
    t_shirt_2:              std::option::Option<egui::TextureHandle>,
    t_shirt:                LoadedImage,
    artwork:                std::option::Option<egui::load::SizedTexture>,
    zoom:                   f32,
    target:                 Vector3<f32>,
    last_drag_pos:          std::option::Option<Vector3<f32>>,
    drag_display_to_tshirt: std::option::Option<Matrix3<f32>>,
    drag_count:             i32
}

/*
impl Default for TShirtCheckerApp<'_> {
    fn default() -> Self {
        Self {
            footer_debug_0:         String::new(),
            footer_debug_1:         String::new(),
            t_shirt_bytes:          include_bytes!("blue_tshirt.png"),         
            t_shirt_colorimage:     None,
            //test_artwork_src:     egui::Image::new(egui::include_image!("hortest.png")) ,
            //test_artwork_src:     egui::Image::new(egui::include_image!("starfest-2024-attendee-v2.png")) ,
            test_artwork_src:       egui::Image::new(egui::include_image!("sf2024-attendee-v1.png")) ,
            t_shirt_2:              None,
            artwork:                None,
            zoom:                   1.0,
            target:                 vector![ 0.50, 0.50, 1.0 ],
            last_drag_pos:          None,
            drag_display_to_tshirt: None,
            drag_count:             0,
        }
    }
}
*/

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

fn v3_to_egui(item : Vector3<f32> ) -> egui::Pos2 {
    egui::Pos2{ x: item.x, y: item.y }
}

fn _eguip_to_v3(item : egui::Pos2 ) -> Vector3<f32> {
    vector![ item.x, item.y, 1.0 ] 
}

fn _eguiv_to_v3(item : egui::Vec2 ) -> Vector3<f32> {
    vector![ item[0], item[1], 1.0 ] 
}

#[cfg(not(target_arch = "wasm32"))]
fn _app_execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || async_std::task::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn _app_execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        Self {
            footer_debug_0:         String::new(),
            footer_debug_1:         String::new(),
            t_shirt_bytes:          include_bytes!("blue_tshirt.png"),         
            t_shirt_colorimage:     None,
            //test_artwork_src:     egui::Image::new(egui::include_image!("hortest.png")) ,
            //test_artwork_src:     egui::Image::new(egui::include_image!("starfest-2024-attendee-v2.png")) ,
            test_artwork_src:       egui::Image::new(egui::include_image!("sf2024-attendee-v1.png")) ,
            t_shirt_2:              None,
            t_shirt:                load_image_from_trusted_source(include_bytes!("blue_tshirt.png"), "blue_shirt", &cc.egui_ctx  ),
            artwork:                None,
            zoom:                   1.0,
            target:                 vector![ 0.50, 0.50, 1.0 ],
            last_drag_pos:          None,
            drag_display_to_tshirt: None,
            drag_count:             0,
        }
    }

    fn do_texture_loads(&mut self, ctx: &egui::Context ) {

        if Option::is_none(&self.artwork) {
            let load_result = self.test_artwork_src.load_for_size( ctx, egui::Vec2{ x: 1.0, y: 1.0 } );
            if Result::is_ok(&load_result) {
                let texture_poll = load_result.unwrap();
                let osize = texture_poll.size();
                let oid = texture_poll.texture_id();
                if Option::is_some( &osize ) && Option::is_some( &oid ) {
                    self.artwork = Some(egui::load::SizedTexture::new(oid.unwrap(), osize.unwrap()));
                }
            }
        }

        if Option::is_none(&self.t_shirt_colorimage) {
            // Prototyping off https://docs.rs/egui/latest/egui/struct.Context.html#method.load_texture
            self.t_shirt_colorimage = Some( egui_extras::image::load_image_bytes( self.t_shirt_bytes ).unwrap() );
            let xsize = self.t_shirt_colorimage.as_ref().unwrap().size[0];
            let ysize = self.t_shirt_colorimage.as_ref().unwrap().size[1];
            let xsize_f = f32::from(i16::try_from(xsize).unwrap());
            let ysize_f = f32::from(i16::try_from(ysize).unwrap());
            let handle: egui::TextureHandle = ctx.load_texture( "my_blue", self.t_shirt_colorimage.as_ref().unwrap().clone(), Default::default() );

            let tshirt_size = handle.size_vec2();
            self.footer_debug_1 = format!("t_shirt size {} {} - {} {} - {} {}", xsize_f, ysize_f, handle.size_vec2().x, handle.size_vec2().y, tshirt_size.x, tshirt_size.y );
            self.t_shirt_2 = Some(handle);
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

    // 
    // Transforms from "t shirt space", where (0,0) is the top
    // left corner of the t shirt image and (1,1) is the bottom
    // right corner of the t-shirt image, to the display.
    // 
    fn tshirt_to_display(&self, ui: &egui::Ui) -> Matrix3<f32> {
        std::assert!(Option::is_some( &self.t_shirt_2 ));
        let panel_size   = ui.available_size_before_wrap();
        let panel_aspect = panel_size[0] / panel_size[1];

        let t_shirt_texture = self.t_shirt_2.as_ref().unwrap();
        let tshirt_size = t_shirt_texture.size_vec2();
        let tshirt_aspect = tshirt_size.x / tshirt_size.y;

        let move_from_center: Matrix3<f32> = 
            matrix![ 1.0,  0.0,  -self.target.x;   
                     0.0,  1.0,  -self.target.y;
                     0.0,  0.0,  1.0 ];
        let move_to_center: Matrix3<f32> = 
            matrix![ 1.0,  0.0,  0.5;
                     0.0,  1.0,  0.5;
                     0.0,  0.0,  1.0 ];
        let scale : Matrix3<f32> =
            matrix![ self.zoom,  0.0,        0.0;   
                     0.0,        self.zoom,  0.0;
                     0.0,        0.0,        1.0 ];

        let scale_centered = move_to_center * scale * move_from_center;

        if panel_aspect > tshirt_aspect {
            // panel is wider than the t-shirt
            let x_width  = panel_size[0] * tshirt_aspect / panel_aspect;
            let x_margin = (panel_size[0] - x_width) / 2.0;
            return matrix![  x_width,    0.0,             x_margin;
                             0.0,        panel_size[1],   0.0;
                             0.0,        0.0,             1.0  ] * scale_centered;
        }
        // panel is higher than the t-shirt
        let y_width  = panel_size[1] / tshirt_aspect * panel_aspect;
        let y_margin = (panel_size[1] - y_width) / 2.0;
        return matrix![  panel_size[0],    0.0,             0.0;
                         0.0,              y_width,         y_margin;
                         0.0,              0.0,             1.0  ] * scale_centered;
    }

    fn art_to_art_space( &self ) -> Matrix3<f32> {
        std::assert!(Option::is_some(&self.artwork ));
        
        let artspace_size   = vector!( 11.0, 14.0 );
        let artspace_aspect = artspace_size.x / artspace_size.y;

        let art_texture     = self.artwork.unwrap();
        let art_size        = art_texture.size;
        let art_aspect      = art_size.x / art_size.y;

        if artspace_aspect > art_aspect {
            // space for art is wider than the artwork
            let x_width  = artspace_size.x * art_aspect / artspace_aspect;
            let x_margin = (artspace_size.x - x_width) / 2.0;
            return matrix![  x_width,    0.0,               x_margin;
                             0.0,        artspace_size.y,   0.0;
                             0.0,        0.0,               1.0  ];
        }
        // panel is higher than the t-shirt
        let y_width  = artspace_size.y / art_aspect * artspace_aspect;
        let y_margin = (artspace_size.y - y_width) / 2.0;
        return matrix![  artspace_size.x,    0.0,             0.0;
                         0.0,                y_width,         y_margin;
                         0.0,                0.0,             1.0  ];
    }

    // 
    // Transforms from "t shirt artwork space", where (0,0) is 
    // the top corner of the artwork and (11.0, 14.0) is the
    // bottom corner, into "t shirt" space.
    // 
    // 11.0 x 14.0 is the working area for the artwork in inches
    // 
    fn art_space_to_tshirt( &self ) -> Matrix3<f32> {
        std::assert!(Option::is_some( &self.t_shirt_2 ));

        let tshirt_texture     = self.t_shirt_2.as_ref().unwrap();
        let tshirt_size        = tshirt_texture.size_vec2();
        let tshirt_aspect      = tshirt_size.x / tshirt_size.y;

        let xcenter = 0.50;  // center artwork mid point for X
        let ycenter = 0.45;  // center artwork 45% down for Y
                            
        let xarea   = 0.48 / 11.0;  // Artwork on 48% of the horizontal image
        // Artwork as 11 x 14 inches, so use that to compute y area
        let yarea   = xarea * tshirt_aspect;

        return matrix![  xarea,          0.0,               xcenter - xarea * 11.0 / 2.0;
                         0.0,            yarea,             ycenter - yarea * 14.0 / 2.0;
                         0.0,            0.0,               1.0 ];
    }

    fn do_central_panel(&mut self, ctx: &egui::Context ) {
        egui::CentralPanel::default().show(ctx, |ui| {

            //if Option::is_some(&self.t_shirt_2 ) {
            if Option::is_some(&self.t_shirt_2 ) {
                let tshirt_to_display = self.tshirt_to_display(ui);
                let t_shirt_texture = self.t_shirt_2.as_ref().unwrap();
                //let t_shirt_texture = self.t_shirt.unwrap();

                let uv0 = egui::Pos2{ x: 0.0, y: 0.0 };
                let uv1 = egui::Pos2{ x: 1.0, y: 1.0 };

                let s0 = v3_to_egui(tshirt_to_display * dvector![0.0, 0.0, 1.0]); 
                let s1 = v3_to_egui(tshirt_to_display * dvector![1.0, 1.0, 1.0]); 

                let (response, painter ) =ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::click_and_drag() );
                painter.image( 
                    t_shirt_texture.id(),
                    egui::Rect::from_min_max(s0, s1 ),
                    egui::Rect::from_min_max(uv0, uv1 ),
                    egui::Color32::WHITE );

                if Option::is_some(&self.artwork) {

                    let art_texture = self.artwork.unwrap();
                    let art_space_to_display = tshirt_to_display * self.art_space_to_tshirt() * self.art_to_art_space();

                    let a0 = v3_to_egui( art_space_to_display * dvector![0.0,  0.0,  1.0] ); 
                    let a1 = v3_to_egui( art_space_to_display * dvector![1.0,  1.0,  1.0] ); 

                    if let Some(pointer_pos) = response.interact_pointer_pos() {
                        let current_drag_pos = vector!( pointer_pos[0], pointer_pos[1], 1.0 );

                        if let Some(last_drag_pos ) = self.last_drag_pos {
                            let display_to_artspace = self.drag_display_to_tshirt.unwrap();
                            let last = display_to_artspace * last_drag_pos;
                            let curr = display_to_artspace * current_drag_pos;
                            self.target = self.target + last - curr;
                        }
                        else {
                            self.drag_display_to_tshirt = Some(tshirt_to_display.try_inverse().unwrap());
                            self.drag_count = self.drag_count + 1
                        }
                        self.last_drag_pos = Some(current_drag_pos);
                    }
                    else {
                        self.last_drag_pos = None;
                        self.drag_display_to_tshirt = None;
                    }

                    if response.hovered() {
                        let zoom_delta_0 = 1.0 + ui.ctx().input(|i| i.smooth_scroll_delta)[1] / 200.0;
                        let zoom_delta_1 = ui.ctx().input(|i| i.zoom_delta());
                        let zoom_delta = if zoom_delta_0 != 1.0 { zoom_delta_0} else {zoom_delta_1};

                        self.zoom = self.zoom * zoom_delta;
                        if self.zoom < 1.0 {
                            self.zoom = 1.0;
                        }
                    }

                    painter.image( 
                        art_texture.id,
                        egui::Rect::from_min_max(a0, a1),
                        egui::Rect::from_min_max(uv0, uv1 ),
                        egui::Color32::WHITE );
                }
            }
        });
    }

    fn gen_status(&self, state : i32 ) -> &str {
        match state {
            0 => "Fail",
            1 => "Warn",
            _ => "Pass"
        }
    }

    fn report_dpi(&self, ui: &mut egui::Ui) {
        if Option::is_some(&self.artwork) {
            let art_texture   = self.artwork.unwrap();
            let top_corner    = self.art_to_art_space() * dvector![ 0.0, 0.0, 1.0 ]; 
            let bot_corner    = self.art_to_art_space() * dvector![ 1.0, 1.0, 1.0 ];
            let dim_in_inches = bot_corner - top_corner;
            let dpi = (art_texture.size.x / dim_in_inches.x) as i32;
            let status : &str = self.gen_status( match dpi {
                0..=74 => 0,
                75 ..=149 => 1,
                _ => 2
            });
            ui.label(mtexts(&format!("{} DPI {}", status, dpi )));
        }
    }

    fn do_right_panel(&mut self, ctx: &egui::Context ) {
        egui::SidePanel::right("stuff")
            .resizable(true)
            .min_width(200.0)
            .show(ctx, |ui| {
             ui.vertical(|ui| {
                let panel_size = ui.available_size_before_wrap();
                self.footer_debug_0 = format!("{} {}", panel_size[0], panel_size[1] );
                ui.heading(mtext("T-Shirt Checker"));
                ui.add_space(10.0);
                self.report_dpi(ui);
            })
        });
    }
}

fn mtext(text: &str) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

fn mtexts(text: &String) -> egui::widget_text::RichText {
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
