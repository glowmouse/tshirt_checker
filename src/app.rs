use web_time::SystemTime;

extern crate nalgebra as na;
use crate::Hsla;
use egui_extras::{Size, StripBuilder};
use na::{dvector, matrix, vector, Matrix3, Vector3};
use std::sync::Arc;

const DEBUG: bool = false;
const TRANSPARENCY_TOGGLE_RATE: u128 = 500;
const TOOL_WIDTH: f32 = 20.0;

fn blue_to_red(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 1024 to adjust the primary color to red.
    let red_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 1024) % (6 * 256),
        s: hsla.s,
        l: hsla.l,
        a: hsla.a,
    };
    red_adjust.into()
}

fn blue_to_dgreen(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 38 to adjust the primary color to dark green
    let dgreen_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 38) % (6 * 256),
        s: hsla.s,
        l: crate::gamma_tables::GAMMA_17[hsla.l as usize],
        a: hsla.a,
    };
    dgreen_adjust.into()
}

fn blue_to_ddgreen(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 38 to adjust the primary color to green, then gamma down saturation.
    let ddgreen_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 38) % (6 * 256),
        s: crate::gamma_tables::GAMMA_30[hsla.s as usize],
        l: crate::gamma_tables::GAMMA_22[hsla.l as usize],
        a: hsla.a,
    };
    ddgreen_adjust.into()
}

fn blue_to_dblue(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 350 to adjust the primary color to dark blue
    let dblue_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 350) % (6 * 256),
        s: crate::gamma_tables::GAMMA_30[hsla.s as usize],
        l: crate::gamma_tables::GAMMA_17[hsla.l as usize],
        a: hsla.a,
    };
    dblue_adjust.into()
}

fn blue_to_burg(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 1024 to adjust the primary color to red.
    let burg_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 439 + 512) % (6 * 256),
        s: hsla.s,
        l: crate::gamma_tables::GAMMA_17[hsla.l as usize],
        a: hsla.a,
    };
    burg_adjust.into()
}

fn correct_alpha_for_tshirt(input: &egui::Color32) -> egui::Color32 {
    let new_a = if input.a() == 0 { 0 } else { 255 };
    egui::Color32::from_rgba_premultiplied(input.r(), input.g(), input.b(), new_a)
}

fn flag_alpha_for_shirt(input: &egui::Color32) -> egui::Color32 {
    let not_binary = input.a() != 0 && input.a() != 255;
    if not_binary {
        egui::Color32::from_rgba_premultiplied(
            255 - input.r(),
            255 - input.g(),
            255 - input.b(),
            255,
        )
    } else {
        *input
    }
}

/// My image abstraction
pub struct LoadedImage {
    uncompressed_image: Arc<egui::ColorImage>,
    texture: egui::TextureHandle,
}

impl Clone for LoadedImage {
    fn clone(&self) -> Self {
        LoadedImage {
            uncompressed_image: self.uncompressed_image.clone(),
            texture: self.texture.clone(),
        }
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

    pub fn size_as_array(&self) -> &[usize; 2] {
        &self.uncompressed_image.size
    }
}

fn compute_bad_tpixels(img: &[egui::Color32]) -> u32 {
    let num_pixels: u32 = img.len().try_into().unwrap();
    let num_bad_pixels: u32 = img
        .iter()
        .filter(|&p| p.a() != 0 && p.a() != 255)
        .count()
        .try_into()
        .unwrap();
    (100 * num_bad_pixels).div_ceil(num_pixels)
}

fn compute_percent_opaque(img: &[egui::Color32]) -> u32 {
    let num_pixels: u32 = img.len().try_into().unwrap();
    let num_opaque_pixels: u32 = img
        .iter()
        .filter(|&p| p.a() > 0)
        .count()
        .try_into()
        .unwrap();
    (100 * num_opaque_pixels).div_ceil(num_pixels)
}

fn load_image_from_untrusted_source(
    bytes: &[u8],
    name: impl Into<String>,
    ctx: &egui::Context,
) -> Result<LoadedImage, String> {
    let uncompressed_image = Arc::new(egui_extras::image::load_image_bytes(bytes)?);
    let handle: egui::TextureHandle =
        ctx.load_texture(name, uncompressed_image.clone(), Default::default());
    Ok(LoadedImage {
        uncompressed_image,
        texture: handle,
    })
}

fn load_image_from_trusted_source(
    bytes: &[u8],
    name: impl Into<String>,
    ctx: &egui::Context,
) -> LoadedImage {
    load_image_from_untrusted_source(bytes, name, ctx).unwrap()
}

fn load_image_from_existing_image(
    existing: &LoadedImage,
    mutator: fn(&egui::Color32) -> egui::Color32,
    name: impl Into<String>,
    ctx: &egui::Context,
) -> LoadedImage {
    let pixels = existing.pixels().iter().map(mutator).collect();
    let size = *existing.size_as_array();
    let uncompressed_image = Arc::new(egui::ColorImage { size, pixels });
    let texture: egui::TextureHandle =
        ctx.load_texture(name, uncompressed_image.clone(), Default::default());

    LoadedImage {
        uncompressed_image,
        texture,
    }
}

fn heat_map_from_image(
    existing: &LoadedImage,
    name: impl Into<String>,
    ctx: &egui::Context,
) -> LoadedImage {
    let in_pixels = existing.pixels();
    //let mut old_out_pixels = Vec::new();
    let xsize = existing.size_as_array()[0];
    let ysize = existing.size_as_array()[1];
    let mut left = in_pixels[0];
    let mut last_row = vec![egui::Color32::BLACK; xsize];
    last_row.copy_from_slice(&in_pixels[0..xsize]);
    let mut x: usize = 0;
    let mut y: usize = 0;
    let heat_x = xsize / 64 + 1;
    let heat_y = ysize / 64 + 1;
    let total_pixels = heat_x * heat_y;
    let mut out_pixels_scalar = std::iter::repeat_with(|| 0)
        .take(total_pixels)
        .collect::<Vec<_>>();

    for pixel in in_pixels {
        let top = last_row[x];
        let (r, g, b, a) = (
            pixel.r() as i32,
            pixel.g() as i32,
            pixel.b() as i32,
            pixel.a() as i32,
        );
        let (rt, gt, bt, at) = (
            top.r() as i32,
            top.g() as i32,
            top.b() as i32,
            top.a() as i32,
        );
        let (rl, gl, bl, al) = (
            left.r() as i32,
            left.g() as i32,
            left.b() as i32,
            left.a() as i32,
        );
        let ld = (rl - r).abs() + (gl - g).abs() + (bl - b).abs() + (al - a).abs();
        let td = (rt - r).abs() + (gt - g).abs() + (bt - b).abs() + (at - a).abs();
        let d = ld + td;

        last_row[x] = *pixel;
        left = *pixel;
        x += 1;
        if x == xsize {
            y += 1;
            x = 0;
        }
        let hx = x / 64;
        let hy = y / 64;
        out_pixels_scalar[hx + hy * heat_x] += d;
        //out_pixels.push( egui::Color32::from_rgba_premultiplied( d, d, d, 255 ));
    }

    let mut max_out_pixel: i32 = 0;
    for scalar in out_pixels_scalar.iter() {
        max_out_pixel = std::cmp::max(max_out_pixel, *scalar);
    }

    let out_pixels = out_pixels_scalar
        .iter()
        .map(|g| {
            let d = 255 * g / max_out_pixel;
            let du8: u8 = d.try_into().unwrap();
            egui::Color32::from_rgba_premultiplied(du8, du8, du8, 255)
        })
        .collect();

    let size = [heat_x, heat_y];
    let uncompressed_image = Arc::new(egui::ColorImage {
        size,
        pixels: out_pixels,
    });
    let texture: egui::TextureHandle =
        ctx.load_texture(name, uncompressed_image.clone(), Default::default());

    LoadedImage {
        uncompressed_image,
        texture,
    }
}

#[derive(PartialEq, Copy, Clone)]
enum ReportStatus {
    Pass,
    Warn,
    Fail,
}

#[derive(PartialEq, Copy, Clone)]
enum ReportTypes {
    Dpi,
    BadTransparency,
    Opaqueness,
    AreaUsed,
}

#[derive(PartialEq, Copy, Clone)]
enum TShirtColors {
    Red,
    DRed,
    Green,
    DGreen,
    Blue,
    DBlue,
}

#[derive(PartialEq, Copy, Clone)]
enum Artwork {
    Artwork0,
    Artwork1,
    Artwork2,
}

pub struct ImageLoad {
    artwork: Artwork,
    image: LoadedImage,
    dependent_data: ArtworkDependentData,
}

#[derive(Eq, Clone)]
pub struct HotSpot {
    strength: u8,
    location: egui::Vec2,
}

impl PartialEq for HotSpot {
    fn eq(&self, other: &Self) -> bool {
        self.strength == other.strength && self.location == other.location
    }
}

use std::cmp::Ordering;

impl PartialOrd for HotSpot {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.strength.cmp(&other.strength))
        Some(other.strength.cmp(&self.strength))
    }
}

impl Ord for HotSpot {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn hot_spots_from_heat_map(heat_map: &LoadedImage) -> Vec<HotSpot> {
    let mut all_hotspots = Vec::new();
    let mut x: u16 = 0;
    let mut y: u16 = 0;
    let xsize = heat_map.size()[0] as u16;
    let xsize_f = heat_map.size()[0];
    let ysize_f = heat_map.size()[1];

    for pixel in heat_map.pixels().iter() {
        all_hotspots.push(HotSpot {
            strength: pixel.r(),
            location: egui::Vec2 {
                x: ((x as f32) + 0.5) / xsize_f,
                y: ((y as f32) + 0.5) / ysize_f,
            },
        });
        x += 1;
        if x == xsize {
            y += 1;
            x = 0;
        }
    }

    all_hotspots.sort();
    let mut chosen_hotspots: Vec<HotSpot> = Vec::new();

    for hotspot in all_hotspots {
        let mut closest_distance = f32::MAX;
        for chosen_hotspot in &chosen_hotspots {
            closest_distance =
                closest_distance.min((chosen_hotspot.location - hotspot.location).length());
        }
        if closest_distance > 0.2 {
            chosen_hotspots.push(hotspot);
        }
        if chosen_hotspots.len() >= 4 {
            break;
        }
    }
    chosen_hotspots
}

pub struct ArtworkDependentData {
    bad_tpixel_percent: u32,
    opaque_percent: u32,
    fixed_artwork: LoadedImage,
    flagged_artwork: LoadedImage,
    _heat_map: LoadedImage,
    top_hot_spots: Vec<HotSpot>,
}

impl ArtworkDependentData {
    fn new(ctx: &egui::Context, artwork: &LoadedImage) -> Self {
        let default_fixed_art: LoadedImage = load_image_from_existing_image(
            artwork,
            correct_alpha_for_tshirt,
            "fixed default art",
            ctx,
        );
        let default_flagged_art: LoadedImage = load_image_from_existing_image(
            artwork,
            flag_alpha_for_shirt,
            "flagged default art",
            ctx,
        );
        let heat_map = heat_map_from_image(artwork, "heatmap", ctx);

        Self {
            bad_tpixel_percent: compute_bad_tpixels(artwork.pixels()),
            opaque_percent: compute_percent_opaque(artwork.pixels()),
            fixed_artwork: default_fixed_art,
            flagged_artwork: default_flagged_art,
            top_hot_spots: hot_spots_from_heat_map(&heat_map),
            _heat_map: heat_map_from_image(artwork, "heatmap", ctx),
        }
    }
}

pub struct ReportTemplate {
    label: String,
    report_tip: String,
    tool_tip: String,
    display_percent: bool,
    metric_to_status: fn(metric: u32) -> ReportStatus,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp {
    artwork_0: LoadedImage,
    artwork_1: LoadedImage,
    artwork_2: LoadedImage,
    art_dependent_data_0: std::option::Option<ArtworkDependentData>,
    art_dependent_data_1: std::option::Option<ArtworkDependentData>,
    art_dependent_data_2: std::option::Option<ArtworkDependentData>,
    selected_art: Artwork,
    footer_debug_0: String,
    footer_debug_1: String,
    blue_t_shirt: LoadedImage,
    red_t_shirt: LoadedImage,
    dgreen_t_shirt: LoadedImage,
    burg_t_shirt: LoadedImage,
    dblue_t_shirt: LoadedImage,
    ddgreen_t_shirt: LoadedImage,
    pass: LoadedImage,
    warn: LoadedImage,
    fail: LoadedImage,
    tool: LoadedImage,
    import: LoadedImage,
    partial_transparency_fix: LoadedImage,
    t_shirt: egui::TextureId,
    zoom: f32,
    target: Vector3<f32>,
    last_drag_pos: std::option::Option<Vector3<f32>>,
    drag_display_to_tshirt: std::option::Option<Matrix3<f32>>,
    drag_count: i32,
    start_time: SystemTime,
    area_used_report: ReportTemplate,
    transparency_report: ReportTemplate,
    opaque_report: ReportTemplate,
    dpi_report: ReportTemplate,
    tool_selected_for: std::option::Option<ReportTypes>,
    tshirt_selected_for: TShirtColors,
    image_loader: Option<std::sync::mpsc::Receiver<Result<ImageLoad, String>>>,
}

fn v3_to_egui(item: Vector3<f32>) -> egui::Pos2 {
    egui::Pos2 {
        x: item.x,
        y: item.y,
    }
}

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

impl TShirtCheckerApp {
    fn is_tool_active(&self, report_type: ReportTypes) -> bool {
        self.tool_selected_for.is_some() && self.tool_selected_for.unwrap() == report_type
    }

    fn do_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
            if DEBUG {
                ui.horizontal(|ui| {
                    ui.label("footer_debug_0: ");
                    ui.label(&self.footer_debug_0);
                });
                ui.horizontal(|ui| {
                    ui.label("footer_debug_1: ");
                    ui.label(&self.footer_debug_1);
                });
                egui::warn_if_debug_build(ui);
            }
            powered_by_egui_and_eframe(ui);
        });
    }

    fn do_load(&mut self, ctx: &egui::Context) {
        let (sender, receiver) = std::sync::mpsc::channel::<Result<ImageLoad, String>>();
        let art_slot = self.selected_art;
        self.image_loader = Some(receiver);
        let thread_ctx = ctx.clone();
        // Execute in another thread
        app_execute(async move {
            let file = rfd::AsyncFileDialog::new().pick_file().await;
            let data: Vec<u8> = file.unwrap().read().await;

            let image = || -> Result<ImageLoad, String> {
                let image = load_image_from_untrusted_source(&data, "loaded_data", &thread_ctx)?;
                let dependent_data = ArtworkDependentData::new(&thread_ctx, &image);
                Ok(ImageLoad {
                    artwork: art_slot,
                    image,
                    dependent_data,
                })
            };

            sender.send(image()).unwrap();
            thread_ctx.request_repaint();
        });
    }

    fn partialt_fix(&mut self, ctx: &egui::Context) {
        let art = self.get_selected_art();
        let fixed_art = load_image_from_existing_image(
            art,
            |p| {
                let new_alpha: u8 = if p.a() < 25 { 0 } else { 255 };
                egui::Color32::from_rgba_premultiplied(p.r(), p.g(), p.b(), new_alpha)
            },
            "fixed_art", // todo, better name...
            ctx,
        );
        let dependent_data = ArtworkDependentData::new(ctx, &fixed_art);
        self.set_artwork(self.selected_art, fixed_art, dependent_data);
    }

    //
    // Transforms from "t shirt space", where (0,0) is the top
    // left corner of the t shirt image and (1,1) is the bottom
    // right corner of the t-shirt image, to the display.
    //
    fn tshirt_to_display(&self, ui: &egui::Ui) -> Matrix3<f32> {
        let panel_size = ui.available_size_before_wrap();
        let panel_aspect = panel_size[0] / panel_size[1];

        let tshirt_size = self.blue_t_shirt.size();
        let tshirt_aspect = tshirt_size.x / tshirt_size.y;

        let move_from_center: Matrix3<f32> = matrix![ 1.0,  0.0,  -self.target.x;
                     0.0,  1.0,  -self.target.y;
                     0.0,  0.0,  1.0 ];
        let move_to_center: Matrix3<f32> = matrix![ 1.0,  0.0,  0.5;
                     0.0,  1.0,  0.5;
                     0.0,  0.0,  1.0 ];
        let scale: Matrix3<f32> = matrix![ self.zoom,  0.0,        0.0;
                     0.0,        self.zoom,  0.0;
                     0.0,        0.0,        1.0 ];

        let scale_centered = move_to_center * scale * move_from_center;

        if panel_aspect > tshirt_aspect {
            // panel is wider than the t-shirt
            let x_width = panel_size[0] * tshirt_aspect / panel_aspect;
            let x_margin = (panel_size[0] - x_width) / 2.0;
            return matrix![  x_width,    0.0,             x_margin;
                             0.0,        panel_size[1],   0.0;
                             0.0,        0.0,             1.0  ]
                * scale_centered;
        }
        // panel is higher than the t-shirt
        let y_width = panel_size[1] / tshirt_aspect * panel_aspect;
        let y_margin = (panel_size[1] - y_width) / 2.0;
        matrix![  panel_size[0],    0.0,             0.0;
                  0.0,              y_width,         y_margin;
                  0.0,              0.0,             1.0  ]
            * scale_centered
    }

    fn art_enum_to_image(&self, artwork: Artwork) -> &LoadedImage {
        match artwork {
            Artwork::Artwork0 => &self.artwork_0,
            Artwork::Artwork1 => &self.artwork_1,
            Artwork::Artwork2 => &self.artwork_2,
        }
    }

    fn art_enum_to_dependent_data(&self, artwork: Artwork) -> &ArtworkDependentData {
        // For now I guess I guarantee, through logic that's hard to reason about
        // that the unwrap always succeeds.  Definately a comments are a code
        // smell moment.
        match artwork {
            Artwork::Artwork0 => self.art_dependent_data_0.as_ref().unwrap(),
            Artwork::Artwork1 => self.art_dependent_data_1.as_ref().unwrap(),
            Artwork::Artwork2 => self.art_dependent_data_2.as_ref().unwrap(),
        }
    }

    fn cache_in_art_dependent_data(&mut self, ctx: &egui::Context, artwork: Artwork) {
        let image: &LoadedImage = self.art_enum_to_image(artwork);

        match artwork {
            Artwork::Artwork0 => {
                if self.art_dependent_data_0.is_none() {
                    self.art_dependent_data_0 = Some(ArtworkDependentData::new(ctx, image));
                }
            }
            Artwork::Artwork1 => {
                if self.art_dependent_data_1.is_none() {
                    self.art_dependent_data_1 = Some(ArtworkDependentData::new(ctx, image));
                }
            }
            Artwork::Artwork2 => {
                if self.art_dependent_data_2.is_none() {
                    self.art_dependent_data_2 = Some(ArtworkDependentData::new(ctx, image));
                }
            }
        }
    }

    fn get_selected_art(&self) -> &LoadedImage {
        self.art_enum_to_image(self.selected_art)
    }

    fn art_to_art_space(&self) -> Matrix3<f32> {
        let artspace_size = vector!(11.0, 14.0);
        let artspace_aspect = artspace_size.x / artspace_size.y;

        let art = self.get_selected_art();
        let art_size = art.size();
        let art_aspect = art_size.x / art_size.y;

        if artspace_aspect > art_aspect {
            // space for art is wider than the artwork
            let x_width = artspace_size.x * art_aspect / artspace_aspect;
            let x_margin = (artspace_size.x - x_width) / 2.0;
            return matrix![  x_width,    0.0,               x_margin;
                             0.0,        artspace_size.y,   0.0;
                             0.0,        0.0,               1.0  ];
        }
        // panel is higher than the t-shirt
        let y_width = artspace_size.y / art_aspect * artspace_aspect;
        let y_margin = (artspace_size.y - y_width) / 2.0;
        matrix![         artspace_size.x,    0.0,             0.0;
                         0.0,                y_width,         y_margin;
                         0.0,                0.0,             1.0  ]
    }

    //
    // Transforms from "t shirt artwork space", where (0,0) is
    // the top corner of the artwork and (11.0, 14.0) is the
    // bottom corner, into "t shirt" space.
    //
    // 11.0 x 14.0 is the working area for the artwork in inches
    //
    fn art_space_to_tshirt(&self) -> Matrix3<f32> {
        let tshirt_size = self.blue_t_shirt.size();
        let tshirt_aspect = tshirt_size.x / tshirt_size.y;

        let xcenter = 0.50; // center artwork mid point for X
        let ycenter = 0.45; // center artwork 45% down for Y

        let xarea = 0.48 / 11.0; // Artwork on 48% of the horizontal image
                                 // Artwork as 11 x 14 inches, so use that to compute y area
        let yarea = xarea * tshirt_aspect;

        matrix![         xarea,          0.0,               xcenter - xarea * 11.0 / 2.0;
                         0.0,            yarea,             ycenter - yarea * 14.0 / 2.0;
                         0.0,            0.0,               1.0 ]
    }

    fn do_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let tshirt_to_display = self.tshirt_to_display(ui);

            let uv0 = egui::Pos2 { x: 0.0, y: 0.0 };
            let uv1 = egui::Pos2 { x: 1.0, y: 1.0 };

            let s0 = v3_to_egui(tshirt_to_display * dvector![0.0, 0.0, 1.0]);
            let s1 = v3_to_egui(tshirt_to_display * dvector![1.0, 1.0, 1.0]);

            let (response, painter) = ui.allocate_painter(
                ui.available_size_before_wrap(),
                egui::Sense::click_and_drag(),
            );
            painter.image(
                self.t_shirt,
                egui::Rect::from_min_max(s0, s1),
                egui::Rect::from_min_max(uv0, uv1),
                egui::Color32::WHITE,
            );

            let art_space_to_display = tshirt_to_display * self.art_space_to_tshirt();
            let art_to_display = art_space_to_display * self.art_to_art_space();

            let a0 = v3_to_egui(art_to_display * dvector![0.0, 0.0, 1.0]);
            let a1 = v3_to_egui(art_to_display * dvector![1.0, 1.0, 1.0]);
            let mut movement_attempted = false;

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let current_drag_pos = vector!(pointer_pos[0], pointer_pos[1], 1.0);

                if let Some(last_drag_pos) = self.last_drag_pos {
                    let display_to_artspace = self.drag_display_to_tshirt.unwrap();
                    let last = display_to_artspace * last_drag_pos;
                    let curr = display_to_artspace * current_drag_pos;
                    self.target = self.target + last - curr;
                    movement_attempted = true;
                } else {
                    self.drag_display_to_tshirt = Some(tshirt_to_display.try_inverse().unwrap());
                    self.drag_count += 1;
                }
                self.last_drag_pos = Some(current_drag_pos);
            } else {
                self.last_drag_pos = None;
                self.drag_display_to_tshirt = None;
            }

            if response.hovered() {
                let zoom_delta_0 = 1.0 + ui.ctx().input(|i| i.smooth_scroll_delta)[1] / 200.0;
                let zoom_delta_1 = ui.ctx().input(|i| i.zoom_delta());
                let zoom_delta = if zoom_delta_0 != 1.0 {
                    zoom_delta_0
                } else {
                    zoom_delta_1
                };
                if zoom_delta != 1.0 {
                    movement_attempted = true;
                }

                self.zoom *= zoom_delta;
                if self.zoom < 1.0 {
                    self.zoom = 1.0;
                }
            }

            let time_in_ms = self.start_time.elapsed().unwrap().as_millis();
            let state = (time_in_ms / TRANSPARENCY_TOGGLE_RATE) % 2;
            let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
            let texture_to_display = if self.is_tool_active(ReportTypes::BadTransparency) {
                match state {
                    0 => dependent_data.flagged_artwork.id(),
                    _ => dependent_data.fixed_artwork.id(),
                }
            } else if self.is_tool_active(ReportTypes::Dpi) {
                dependent_data.fixed_artwork.id()
            } else {
                self.get_selected_art().id()
            };

            painter.image(
                texture_to_display,
                egui::Rect::from_min_max(a0, a1),
                egui::Rect::from_min_max(uv0, uv1),
                egui::Color32::WHITE,
            );

            if self.is_tool_active(ReportTypes::Dpi) {
                let cycle = time_in_ms / TRANSPARENCY_TOGGLE_RATE / 10;
                let slot = cycle % (dependent_data.top_hot_spots.len() as u128);
                let hot_spot = &dependent_data.top_hot_spots[slot as usize];
                let art_location = vector![hot_spot.location.x, hot_spot.location.y, 1.0];
                let art_to_tshirt = self.art_space_to_tshirt() * self.art_to_art_space();
                let display_location = art_to_tshirt * art_location;

                // need to make modifications to self after dependent_data borrow is done.
                if !movement_attempted {
                    self.zoom = 10.0;
                    self.target = display_location;
                } else {
                    // deselect tool if the user is trying to move or zoom.
                    self.tool_selected_for = None;
                }
            }

            if self.is_tool_active(ReportTypes::AreaUsed) {
                let art_space_border = vec![
                    v3_to_egui(art_space_to_display * dvector![0.0, 0.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![11.0, 0.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![11.0, 14.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![0.0, 14.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![0.0, 0.0, 1.0]),
                ];

                let dash_dim = (art_space_to_display * dvector![0.2, 0.05, 1.0])
                    - (art_space_to_display * dvector![0.0, 0.0, 1.0]);
                let dash_length = dash_dim.x;
                let dash_width = dash_dim.y;
                let gap_length = dash_length;

                // animate with 3 cycles
                let cycle =
                    (time_in_ms % (TRANSPARENCY_TOGGLE_RATE * 3)) / TRANSPARENCY_TOGGLE_RATE;
                let offset: f32 = (cycle as f32) / 3.0 * (dash_length + gap_length);
                let stroke_1 =
                    egui::Stroke::new(dash_width, egui::Color32::from_rgb(200, 200, 200));

                painter.add(egui::Shape::dashed_line_with_offset(
                    &art_space_border,
                    stroke_1,
                    &[dash_length],
                    &[gap_length],
                    offset,
                ));
            }
        });
    }

    fn gen_status_icon(&self, status: ReportStatus) -> egui::Image<'_> {
        egui::Image::from_texture(match status {
            ReportStatus::Fail => self.fail.texture_handle(),
            ReportStatus::Warn => self.warn.texture_handle(),
            ReportStatus::Pass => self.pass.texture_handle(),
        })
        .max_width(25.0)
    }

    fn tshirt_enum_to_image(&self, color: TShirtColors) -> &LoadedImage {
        match color {
            TShirtColors::Red => &self.red_t_shirt,
            TShirtColors::DRed => &self.burg_t_shirt,
            TShirtColors::Green => &self.dgreen_t_shirt,
            TShirtColors::DGreen => &self.ddgreen_t_shirt,
            TShirtColors::Blue => &self.blue_t_shirt,
            TShirtColors::DBlue => &self.dblue_t_shirt,
        }
    }

    fn handle_tshirt_button(&mut self, ui: &mut egui::Ui, color: TShirtColors) {
        let image: &LoadedImage = self.tshirt_enum_to_image(color);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(80.0);
        let is_selected = self.tshirt_selected_for == color;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            self.t_shirt = image.id();
            self.tshirt_selected_for = color;
        }
    }

    fn handle_art_button(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, artwork: Artwork) {
        let image: &LoadedImage = self.art_enum_to_image(artwork);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(80.0);
        let is_selected = self.selected_art == artwork;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            self.cache_in_art_dependent_data(ctx, artwork);
            self.selected_art = artwork;
            self.tool_selected_for = None; // Reset tool selection.
        }
    }

    fn compute_dpi(&self) -> u32 {
        let top_corner = self.art_to_art_space() * dvector![0.0, 0.0, 1.0];
        let bot_corner = self.art_to_art_space() * dvector![1.0, 1.0, 1.0];
        let dim_in_inches = bot_corner - top_corner;
        let art = &self.get_selected_art();
        (art.size().x / dim_in_inches.x) as u32
    }

    fn dpi_to_status(dpi: u32) -> ReportStatus {
        match dpi {
            0..=199 => ReportStatus::Fail,
            200..=299 => ReportStatus::Warn,
            _ => ReportStatus::Pass,
        }
    }

    fn compute_badtransparency_pixels(&self) -> u32 {
        let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
        dependent_data.bad_tpixel_percent
    }

    fn bad_transparency_to_status(bad_transparency_pixels: u32) -> ReportStatus {
        match bad_transparency_pixels {
            0 => ReportStatus::Pass,
            _ => ReportStatus::Fail,
        }
    }

    fn compute_area_used(&self) -> u32 {
        let top_corner = self.art_to_art_space() * dvector![0.0, 0.0, 1.0];
        let bot_corner = self.art_to_art_space() * dvector![1.0, 1.0, 1.0];
        let dim_in_inches = bot_corner - top_corner;
        let area_used = 100.0 * dim_in_inches[0] * dim_in_inches[1] / (11.0 * 14.0);
        area_used as u32
    }

    fn area_used_to_status(area_used: u32) -> ReportStatus {
        match area_used {
            0..=50 => ReportStatus::Fail,
            51..=90 => ReportStatus::Warn,
            _ => ReportStatus::Pass,
        }
    }

    fn compute_opaque_percentage(&self) -> u32 {
        let area_used = self.compute_area_used();
        let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
        area_used * dependent_data.opaque_percent / 100
    }

    fn opaque_to_status(opaque_area: u32) -> ReportStatus {
        match opaque_area {
            0..=49 => ReportStatus::Pass,
            50..=74 => ReportStatus::Warn,
            _ => ReportStatus::Fail,
        }
    }

    fn report_type_to_template(&self, report_type: ReportTypes) -> &ReportTemplate {
        match report_type {
            ReportTypes::Dpi => &self.dpi_report,
            ReportTypes::AreaUsed => &self.area_used_report,
            ReportTypes::BadTransparency => &self.transparency_report,
            ReportTypes::Opaqueness => &self.opaque_report,
        }
    }

    fn report_metric(&mut self, ui: &mut egui::Ui, report_type: ReportTypes, metric: u32) {
        ui.horizontal(|ui| {
            StripBuilder::new(ui)
                .size(Size::exact(25.0))
                .size(Size::exact(140.0))
                .size(Size::exact(40.0))
                .size(Size::exact(15.0))
                .size(Size::exact(TOOL_WIDTH))
                .horizontal(|mut strip| {
                    let report = self.report_type_to_template(report_type);
                    let status = (report.metric_to_status)(metric);
                    let status_icon = self.gen_status_icon(status);
                    let tool_tip = report.tool_tip.clone();
                    let report_tip = report.report_tip.clone();

                    strip.cell(|ui| {
                        ui.add(status_icon).on_hover_text(&report_tip);
                    });
                    strip.cell(|ui| {
                        ui.label(mtexts(&report.label.to_string()))
                            .on_hover_text(&report_tip);
                    });
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(mtexts(&format!("{}", metric)))
                                .on_hover_text(&report_tip);
                        });
                    });
                    let cell_string = (if report.display_percent { "%" } else { "" }).to_string();
                    strip.cell(|ui| {
                        ui.label(mtexts(&cell_string));
                    });
                    strip.cell(|ui| {
                        if status != ReportStatus::Pass {
                            let is_selected = self.is_tool_active(report_type);
                            if ui
                                .add(
                                    egui::widgets::ImageButton::new(
                                        egui::Image::from_texture(self.tool.texture_handle())
                                            .max_width(TOOL_WIDTH)
                                            .bg_fill(egui::Color32::WHITE),
                                    )
                                    .selected(is_selected),
                                )
                                .on_hover_text(tool_tip)
                                .clicked()
                            {
                                self.tool_selected_for =
                                    if is_selected { None } else { Some(report_type) };
                                self.start_time = SystemTime::now();
                            }
                        }
                    });
                });
        });
    }

    fn set_artwork(
        &mut self,
        slot: Artwork,
        image: LoadedImage,
        dependent_data: ArtworkDependentData,
    ) {
        match slot {
            Artwork::Artwork0 => {
                self.artwork_0 = image;
                self.art_dependent_data_0 = Some(dependent_data);
            }
            Artwork::Artwork1 => {
                self.artwork_1 = image;
                self.art_dependent_data_1 = Some(dependent_data);
            }
            Artwork::Artwork2 => {
                self.artwork_2 = image;
                self.art_dependent_data_2 = Some(dependent_data);
            }
        }
    }

    fn do_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("stuff")
            .resizable(true)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.vertical_centered(|ui| {
                        ui.heading(
                            egui::widget_text::RichText::from("T-Shirt Art Checker").size(30.0),
                        )
                    });
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);

                    self.report_metric(ui, ReportTypes::Dpi, self.compute_dpi());
                    self.report_metric(ui, ReportTypes::AreaUsed, self.compute_area_used());
                    self.report_metric(
                        ui,
                        ReportTypes::Opaqueness,
                        self.compute_opaque_percentage(),
                    );
                    self.report_metric(
                        ui,
                        ReportTypes::BadTransparency,
                        self.compute_badtransparency_pixels(),
                    );

                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    //let max_size = egui::Vec2{ x: 30.0, y: 30.0 };
                    ui.horizontal(|ui| {
                        self.handle_tshirt_button(ui, TShirtColors::Red);
                        self.handle_tshirt_button(ui, TShirtColors::Green);
                        self.handle_tshirt_button(ui, TShirtColors::Blue);
                    });
                    ui.horizontal(|ui| {
                        self.handle_tshirt_button(ui, TShirtColors::DRed);
                        self.handle_tshirt_button(ui, TShirtColors::DGreen);
                        self.handle_tshirt_button(ui, TShirtColors::DBlue);
                    });
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        self.handle_art_button(ctx, ui, Artwork::Artwork0);
                        self.handle_art_button(ctx, ui, Artwork::Artwork1);
                        self.handle_art_button(ctx, ui, Artwork::Artwork2);
                    });
                    ui.add_space(5.0);
                    ui.separator();
                    ui.add_space(5.0);
                    //let image: &LoadedImage = self.art_enum_to_image(artwork);
                    //let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(80.0);
                    //let is_selected = self.selected_art == artwork;
                    //if ui
                    //    .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
                    //    .clicked()
                    ui.horizontal(|ui| {
                        let import_icon = egui::Image::from_texture(self.import.texture_handle())
                            .max_width(80.0)
                            .bg_fill(egui::Color32::WHITE);
                        if ui
                            .add(egui::widgets::ImageButton::new(import_icon))
                            .on_hover_text("Import an image to the selected artwork slot.")
                            .clicked()
                        {
                            self.do_load(ctx);
                        }
                        let partialt_icon = egui::Image::from_texture(
                            self.partial_transparency_fix.texture_handle(),
                        )
                        .max_width(80.0)
                        .bg_fill(egui::Color32::WHITE);
                        if ui
                            .add(egui::widgets::ImageButton::new(partialt_icon))
                            .on_hover_text("Fix partial transparency problems by mapping all alpha values to 0 or 1.")
                            .clicked()
                        {
                            self.partialt_fix(ctx);
                        }
                    });
                })
            });
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let blue_shirt: LoadedImage = load_image_from_trusted_source(
            include_bytes!("blue_tshirt.png"),
            "blue_shirt",
            &cc.egui_ctx,
        );
        let artwork_0: LoadedImage = load_image_from_trusted_source(
            include_bytes!("test_artwork.png"),
            "artwork_0",
            &cc.egui_ctx,
        );
        let artwork_1: LoadedImage = load_image_from_trusted_source(
            include_bytes!("sf2024-attendee-v1.png"),
            "artwork_1",
            &cc.egui_ctx,
        );
        let artwork_2: LoadedImage = load_image_from_trusted_source(
            include_bytes!("sf2024-attendee-v2.png"),
            "artwork_2",
            &cc.egui_ctx,
        );
        let red_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, blue_to_red, "red_shirt", &cc.egui_ctx);
        let dgreen_shirt: LoadedImage = load_image_from_existing_image(
            &blue_shirt,
            blue_to_dgreen,
            "dgreen_shirt",
            &cc.egui_ctx,
        );
        let ddgreen_shirt: LoadedImage = load_image_from_existing_image(
            &blue_shirt,
            blue_to_ddgreen,
            "ddgreen_shirt",
            &cc.egui_ctx,
        );
        let dblue_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, blue_to_dblue, "dblue_shirt", &cc.egui_ctx);

        let burg_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, blue_to_burg, "burg_shirt", &cc.egui_ctx);
        let default_shirt = red_shirt.id();
        let pass: LoadedImage =
            load_image_from_trusted_source(include_bytes!("pass.png"), "pass", &cc.egui_ctx);
        let warn: LoadedImage =
            load_image_from_trusted_source(include_bytes!("warn.png"), "warn", &cc.egui_ctx);
        let fail: LoadedImage =
            load_image_from_trusted_source(include_bytes!("fail.png"), "fail", &cc.egui_ctx);
        let tool: LoadedImage =
            load_image_from_trusted_source(include_bytes!("tool.png"), "tool", &cc.egui_ctx);
        let import: LoadedImage = load_image_from_trusted_source(
            include_bytes!("import_80x80.png"),
            "import",
            &cc.egui_ctx,
        );
        let partial_transparency_fix: LoadedImage = load_image_from_trusted_source(
            include_bytes!("partialt_80x80.png"),
            "partialt",
            &cc.egui_ctx,
        );

        let dpi_report = ReportTemplate {
            label: "DPI".to_string(),
            report_tip: "Ideally, artwork for T-Shirts should be Print Quality - 300 DPI or more. Medium Quality (200 to 299 DPI) is probably okay. Below 200 DPI pixalation may be noticable.".to_string(),
            tool_tip: "Show close ups of areas where artwork might look pixelly.\nTurn off the tool or move the T-Shirt to exit.".to_string(),
            display_percent: false,
            metric_to_status: TShirtCheckerApp::dpi_to_status,
        };
        let area_used_report = ReportTemplate {
            label: "Area Used".to_string(),
            report_tip: "Artwork is usually printed on an 11 inch by 14 inch area of the T-Shirt.  The report shows how much of that printable area the art is currently filling.  There's no rule that says art has to use all of the available area, but it's nice to know how much available area there is.".to_string(),
            tool_tip: "Show the maximum boundary of the printable area on the T-Shirt.".to_string(),
            display_percent: true,
            metric_to_status: TShirtCheckerApp::area_used_to_status,
        };
        let transparency_report = ReportTemplate {
            label: "Partial\nTransparency".to_string(),
            report_tip: "The processed used to print T-Shirt artwork doesn't support partial transparency.  Either the artwork is being printed (100% transparecy) or the T-Shirt is showing through (0% transparency) - there's nothing in between.  For best results, fix partial transparency problems in your art package of choice.".to_string(),
            tool_tip: "Show areas of the artwork where there's partial transparency of some kind.".to_string(),
            display_percent: true,
            metric_to_status: TShirtCheckerApp::bad_transparency_to_status,
        };
        let opaque_report = ReportTemplate {
            label: "Bib Score".to_string(),
            report_tip: "T-Shirt artwork shouldn't cover all the printable area.  The more area the artwork covers, the more the T-Shirt will feel like a pastic bib you'd put on a baby for meal time.  For best results the artwork have transparent areas where the T-Shirt will show through, and work with the T-Shirt color.".to_string(),
            tool_tip: "TODO: have tool do something.".to_string(),
            display_percent: true,
            metric_to_status: TShirtCheckerApp::opaque_to_status,
        };

        Self {
            art_dependent_data_0: Some(ArtworkDependentData::new(&cc.egui_ctx, &artwork_0)),
            art_dependent_data_1: None,
            art_dependent_data_2: None,
            selected_art: Artwork::Artwork0,
            footer_debug_0: String::new(),
            footer_debug_1: String::new(),
            blue_t_shirt: blue_shirt,
            red_t_shirt: red_shirt,
            dgreen_t_shirt: dgreen_shirt,
            burg_t_shirt: burg_shirt,
            dblue_t_shirt: dblue_shirt,
            ddgreen_t_shirt: ddgreen_shirt,
            t_shirt: default_shirt,
            pass,
            warn,
            fail,
            tool,
            import,
            partial_transparency_fix,
            zoom: 1.0,
            target: vector![0.50, 0.50, 1.0],
            last_drag_pos: None,
            drag_display_to_tshirt: None,
            drag_count: 0,
            start_time: SystemTime::now(),
            area_used_report,
            dpi_report,
            opaque_report,
            transparency_report,
            tool_selected_for: None,
            tshirt_selected_for: TShirtColors::Red,
            artwork_0,
            artwork_1,
            artwork_2,
            image_loader: None,
        }
    }
}

fn mtexts(text: &String) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

impl eframe::App for TShirtCheckerApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.footer_debug_0 = format!("time {}", self.start_time.elapsed().unwrap().as_millis());
        if self.image_loader.is_some() {
            let rcv = self.image_loader.as_ref().unwrap();
            let data_attempt = rcv.try_recv();
            if data_attempt.is_ok() {
                let loaded_result = data_attempt.unwrap();
                match loaded_result {
                    Err(e) => {
                        self.footer_debug_1 = format!("Error: {}", e);
                    }
                    Ok(f) => {
                        self.set_artwork(f.artwork, f.image, f.dependent_data);
                    }
                }
                self.image_loader = None;
            }
        }
        self.do_bottom_panel(ctx);
        self.do_right_panel(ctx);
        self.do_central_panel(ctx);
        if self.is_tool_active(ReportTypes::BadTransparency)
            || self.is_tool_active(ReportTypes::AreaUsed)
            || self.is_tool_active(ReportTypes::Dpi)
        {
            let time_in_ms = self.start_time.elapsed().unwrap().as_millis();
            let next_epoch =
                (time_in_ms / TRANSPARENCY_TOGGLE_RATE + 1) * TRANSPARENCY_TOGGLE_RATE + 1;
            let time_to_wait = next_epoch - time_in_ms;

            ctx.request_repaint_after(std::time::Duration::from_millis(
                time_to_wait.try_into().unwrap(),
            ))
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
