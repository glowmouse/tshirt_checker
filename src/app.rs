
extern crate nalgebra as na;
use na::{Matrix3, matrix, dvector, vector, Vector3};

const DEBUG: bool = false;

pub struct HSLA {
  h:      u16,
  s:      u8,
  l:      u8,
  a:      u8,
}

impl From<egui::Color32> for HSLA {
  fn from( item: egui::Color32 ) -> Self {
    let r : i32 = i32::from(item.r());
    let g : i32 = i32::from(item.g());
    let b : i32 = i32::from(item.b());

    let min : i32 = core::cmp::min( core::cmp::min( r, g ), b );
    let max : i32 = core::cmp::max( core::cmp::max( r, g ), b );

    let l : i32 = (min + max ) / 2;

    if min == max {
      return HSLA{ h : 0, s: 0, l: u8::try_from(l).unwrap(), a: item.a() }; 
    }


    let half : i32 = 128;
    let two  : i32 = 512;
    let four : i32 = 1024;

    let s2 : i32 = 
        if l <= half          { (( max - min ) << 8 ) / ( max + min )       }
        else                  { (( max - min ) << 8 ) / ( two - max - min ) };

    let s = if s2 == 256 { 255 } else { s2 };

    let ht : i32 =
        if      r == max      {         (( g - b ) << 8 ) / (max - min ) }
        else if g == max      { two  +  (( b - r ) << 8 ) / (max - min ) }
        else                  { four +  (( r - g ) << 8 ) / (max - min ) };

    let h = (ht + 256*6 ) % ( 256 * 6 );

    std::assert!( l >= 0 && l < 256 );
    std::assert!( s >= 0 );
    std::assert!( s <= 256 );
    std::assert!( s < 256 );
    std::assert!( h >= 0 );
    std::assert!( h <= 256 * 6 ); 

    HSLA{ h : u16::try_from(h).unwrap(), s : u8::try_from(s).unwrap(), l : u8::try_from(l).unwrap(), a : item.a() }
  }
}

impl Into<egui::Color32> for HSLA {

  // https://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/

  fn into(self) -> egui::Color32 {
    if self.s == 0 {
      return egui::Color32::from_rgba_unmultiplied( self.l, self.l, self.l, self.a );
    }
    let half : i32 = 128;
    let one  : i32 = 256;
    let h    : i32 = i32::from( self.h );
    let s    : i32 = i32::from( self.s );
    let l    : i32 = i32::from( self.l );

    let temp1 : i32 = 
        if l < half    { (l * ( one + s )) >> 8  }
        else           { l + s - (( l * s ) >> 8 ) };
    let temp2 : i32 = 2 * l - temp1;

    fn hue_to_rgb_2( t1: i32, t2: i32, harg : i32 ) -> i32 {
      let h = harg % ( 6 * 256 );
      let one   : i32 = 256;
      let three : i32 = 256*3;
      let four  : i32 = 256*4;
      if        h < one       { t2 + (t1 - t2 ) * h / 256 }
      else if   h < three     { t1 }
      else if   h < four      { t2 + (t1 - t2 ) * ( four - h ) / 256 }
      else                    { t2 } 
    }

    fn hue_to_rgb( t1 : i32, t2 : i32, h: i32 ) -> u8 {
      // we sometimes get small negatives.  skill issue/ bug.
      let tmp = std::cmp::min(255, std::cmp::max( 0, hue_to_rgb_2( t1, t2, h )));
      u8::try_from( tmp ).unwrap()
    }

    let r = hue_to_rgb( temp1, temp2, h + 512 );
    let g = hue_to_rgb( temp1, temp2, h         );
    let b = hue_to_rgb( temp1, temp2, h - 512 );

    return egui::Color32::from_rgba_unmultiplied( r, g, b, self.a );
  }
}

fn blue_to_red( input : egui::Color32 ) -> egui::Color32 {
  let hsla = HSLA::from( input );
  // -324 adjusts the original blue green shirt to a primary color
  // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
  // 1024 to adjust the primary color to red.
  let red_adjust = HSLA{ h: ( hsla.h + 6 * 256 - 324 + 1024 ) % ( 6 * 256 ), s: hsla.s, l : hsla.l, a: hsla.a };
  red_adjust.into()
} 

fn int_gamma( input : u8, gamma : f32 ) -> u8 {
    let finput = ( input as f32 ) / 255.0;
    let fout = f32::powf( finput, gamma ) * 255.0;
    fout as u8
}

fn blue_to_dgreen( input : egui::Color32 ) -> egui::Color32 {
  let hsla = HSLA::from( input );
  // -324 adjusts the original blue green shirt to a primary color
  // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
  // 1024 to adjust the primary color to red.
  let dgreen_adjust = HSLA{ h: ( hsla.h + 6 * 256 - 324 + 38 ) % ( 6 * 256 ), s: hsla.s, l : int_gamma(hsla.l, 1.7), a: hsla.a };
  dgreen_adjust.into()
}

fn blue_to_burg( input : egui::Color32 ) -> egui::Color32 {
  let hsla = HSLA::from( input );
  // -324 adjusts the original blue green shirt to a primary color
  // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
  // 1024 to adjust the primary color to red.
  let burg_adjust = HSLA{ h: ( hsla.h + 6 * 256 - 324 + 439 + 512 ) % ( 6 * 256 ), s: hsla.s, l : int_gamma(hsla.l, 1.7), a: hsla.a };
  burg_adjust.into()
} 

/// My image abstraction
pub struct LoadedImage {
    uncompressed_image:     egui::ColorImage,
    texture:                egui::TextureHandle,
}

impl Clone for LoadedImage {
  fn clone(&self) -> Self {
    LoadedImage{ uncompressed_image: self.uncompressed_image.clone(), texture: self.texture.clone() }
  }
}

impl LoadedImage {
    pub fn id(&self) -> egui::TextureId {
      self.texture.id()
    }

    pub fn texture_handle(&self) -> &egui::TextureHandle {
      &self.texture
    }

    pub fn size(&self) -> egui::Vec2 {
      self.texture.size_vec2()
    }

    pub fn pixels(&self) -> &Vec<egui::Color32> {
      &self.uncompressed_image.pixels
    }

    pub fn size_as_array(&self) -> &[usize; 2 ] {
      &self.uncompressed_image.size
    }
}



fn load_image_from_trusted_source( bytes : &[u8],  name: impl Into<String>, ctx: &egui::Context ) -> LoadedImage 
{
    let uncompressed_image = egui_extras::image::load_image_bytes( bytes ).unwrap();
    let handle: egui::TextureHandle = ctx.load_texture( name, uncompressed_image.clone(), Default::default() );
    LoadedImage{ uncompressed_image: uncompressed_image, texture: handle }
}

fn load_image_from_existing_image( 
  existing: &LoadedImage, 
  mutator: fn( egui::Color32 ) -> egui::Color32, 
  name: impl Into<String>, ctx: &egui::Context ) -> LoadedImage 
{
  let mut new_image : Vec<egui::Color32> = vec![];

  for color in existing.pixels().iter() {
    new_image.push( mutator( *color ));
  } 

  let uncompressed_image = egui::ColorImage { size: existing.size_as_array().clone(), pixels: new_image };
  let handle: egui::TextureHandle = ctx.load_texture( name, uncompressed_image.clone(), Default::default() );
  LoadedImage{ uncompressed_image: uncompressed_image, texture: handle }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp<'a> {
    footer_debug_0:         String,
    footer_debug_1:         String,
    test_artwork_src:       egui::Image<'a>,
    blue_t_shirt:           LoadedImage,
    red_t_shirt:            LoadedImage,
    dgreen_t_shirt:         LoadedImage,
    burg_t_shirt:           LoadedImage,
    t_shirt:                egui::TextureId,
    artwork:                std::option::Option<egui::load::SizedTexture>,
    zoom:                   f32,
    target:                 Vector3<f32>,
    last_drag_pos:          std::option::Option<Vector3<f32>>,
    drag_display_to_tshirt: std::option::Option<Matrix3<f32>>,
    drag_count:             i32
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
        let blue_shirt : LoadedImage = load_image_from_trusted_source(include_bytes!("blue_tshirt.png"), "blue_shirt", &cc.egui_ctx  );
        let red_shirt : LoadedImage = load_image_from_existing_image( &blue_shirt, blue_to_red, "red_shirt", &cc.egui_ctx ); 
        let dgreen_shirt: LoadedImage = load_image_from_existing_image( &blue_shirt, blue_to_dgreen, "dgreen_shirt", &cc.egui_ctx ); 
        let burg_shirt: LoadedImage = load_image_from_existing_image( &blue_shirt, blue_to_burg, "burg_shirt", &cc.egui_ctx ); 
        let default_shirt = red_shirt.id();

        Self {
            footer_debug_0:         String::new(),
            footer_debug_1:         String::new(),
            //test_artwork_src:     egui::Image::new(egui::include_image!("hortest.png")) ,
            //test_artwork_src:     egui::Image::new(egui::include_image!("starfest-2024-attendee-v2.png")) ,
            test_artwork_src:       egui::Image::new(egui::include_image!("sf2024-attendee-v1.png")) ,
            blue_t_shirt:           blue_shirt,
            red_t_shirt:            red_shirt,
            dgreen_t_shirt:         dgreen_shirt,
            burg_t_shirt:           burg_shirt,
            t_shirt:                default_shirt,           
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
    }

    fn do_bottom_panel(&self, ctx: &egui::Context ) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
            if DEBUG {
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
                egui::warn_if_debug_build(ui);
            }
            powered_by_egui_and_eframe(ui);
        });
    }

    // 
    // Transforms from "t shirt space", where (0,0) is the top
    // left corner of the t shirt image and (1,1) is the bottom
    // right corner of the t-shirt image, to the display.
    // 
    fn tshirt_to_display(&self, ui: &egui::Ui) -> Matrix3<f32> {
        let panel_size   = ui.available_size_before_wrap();
        let panel_aspect = panel_size[0] / panel_size[1];

        let tshirt_size = self.blue_t_shirt.size();
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
        let tshirt_size        = self.blue_t_shirt.size();
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
            if true {
                let tshirt_to_display = self.tshirt_to_display(ui);

                let uv0 = egui::Pos2{ x: 0.0, y: 0.0 };
                let uv1 = egui::Pos2{ x: 1.0, y: 1.0 };

                let s0 = v3_to_egui(tshirt_to_display * dvector![0.0, 0.0, 1.0]); 
                let s1 = v3_to_egui(tshirt_to_display * dvector![1.0, 1.0, 1.0]); 

                let (response, painter ) =ui.allocate_painter(ui.available_size_before_wrap(), egui::Sense::click_and_drag() );
                painter.image( 
                    self.t_shirt,
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
                //let max_size = egui::Vec2{ x: 30.0, y: 30.0 };
                ui.horizontal(|ui| {
                    if (ui.add(egui::widgets::ImageButton::new( egui::Image::from_texture( self.burg_t_shirt.texture_handle()   ).max_width(90.0)))).clicked() {
                        self.t_shirt = self.burg_t_shirt.id();
                    }
                    if ui.add(egui::widgets::ImageButton::new( egui::Image::from_texture( self.dgreen_t_shirt.texture_handle() ).max_width(90.0))).clicked() {
                        self.t_shirt = self.dgreen_t_shirt.id();
                    }
                });
                ui.horizontal(|ui| {
                    if ui.add(egui::widgets::ImageButton::new( egui::Image::from_texture( self.blue_t_shirt.texture_handle()   ).max_width(90.0))).clicked() {
                        self.t_shirt = self.blue_t_shirt.id();
                    }
                    if ui.add(egui::widgets::ImageButton::new( egui::Image::from_texture( self.red_t_shirt.texture_handle()    ).max_width(90.0))).clicked() {
                        self.t_shirt = self.red_t_shirt.id();
                    }
                });

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
