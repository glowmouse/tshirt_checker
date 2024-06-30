use web_time::SystemTime;

extern crate nalgebra as na;
use egui_extras::{Size, StripBuilder};
use na::{dvector, matrix, vector, Matrix3, Vector3};
use std::sync::Arc;

const DEBUG: bool = false;
const TRANSPARENCY_TOGGLE_RATE: u128 = 500;
const TOOL_WIDTH: f32 = 25.0;

pub struct Hsla {
    h: u16,
    s: u8,
    l: u8,
    a: u8,
}

impl From<&egui::Color32> for Hsla {
    fn from(item: &egui::Color32) -> Self {
        let r: i32 = i32::from(item.r());
        let g: i32 = i32::from(item.g());
        let b: i32 = i32::from(item.b());

        let min: i32 = core::cmp::min(core::cmp::min(r, g), b);
        let max: i32 = core::cmp::max(core::cmp::max(r, g), b);

        let l: i32 = (min + max) / 2;

        if min == max {
            return Hsla {
                h: 0,
                s: 0,
                l: u8::try_from(l).unwrap(),
                a: item.a(),
            };
        }

        let half: i32 = 128;
        let two: i32 = 512;
        let four: i32 = 1024;

        let s2: i32 = if l <= half {
            ((max - min) << 8) / (max + min)
        } else {
            ((max - min) << 8) / (two - max - min)
        };

        let s = if s2 == 256 { 255 } else { s2 };

        let ht: i32 = if r == max {
            ((g - b) << 8) / (max - min)
        } else if g == max {
            two + ((b - r) << 8) / (max - min)
        } else {
            four + ((r - g) << 8) / (max - min)
        };

        let h = (ht + 256 * 6) % (256 * 6);

        std::assert!(h >= 0);
        std::assert!(h <= 256 * 6);

        Hsla {
            h: u16::try_from(h).unwrap(),
            s: u8::try_from(s).unwrap(),
            l: u8::try_from(l).unwrap(),
            a: item.a(),
        }
    }
}

impl From<Hsla> for egui::Color32 {
    // https://www.niwa.nu/2013/05/math-behind-colorspace-conversions-rgb-hsl/

    fn from(val: Hsla) -> Self {
        if val.s == 0 {
            return egui::Color32::from_rgba_premultiplied(val.l, val.l, val.l, val.a);
        }
        let half: i32 = 128;
        let one: i32 = 256;
        let h: i32 = i32::from(val.h);
        let s: i32 = i32::from(val.s);
        let l: i32 = i32::from(val.l);

        let temp1: i32 = if l < half {
            (l * (one + s)) >> 8
        } else {
            l + s - ((l * s) >> 8)
        };
        let temp2: i32 = 2 * l - temp1;

        fn hue_to_rgb_2(t1: i32, t2: i32, harg: i32) -> i32 {
            let h = harg % (6 * 256);
            let one: i32 = 256;
            let three: i32 = 256 * 3;
            let four: i32 = 256 * 4;
            if h < one {
                t2 + (t1 - t2) * h / 256
            } else if h < three {
                t1
            } else if h < four {
                t2 + (t1 - t2) * (four - h) / 256
            } else {
                t2
            }
        }

        fn hue_to_rgb(t1: i32, t2: i32, h: i32) -> u8 {
            // we sometimes get small negatives.  skill issue/ bug.
            let tmp = hue_to_rgb_2(t1, t2, h).clamp(0, 255);
            u8::try_from(tmp).unwrap()
        }

        let r = hue_to_rgb(temp1, temp2, h + 512);
        let g = hue_to_rgb(temp1, temp2, h);
        let b = hue_to_rgb(temp1, temp2, h - 512);

        egui::Color32::from_rgba_premultiplied(r, g, b, val.a)
    }
}

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

// Gamma correction tables.
const GAMMA_17: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7,
    7, 7, 8, 8, 9, 9, 10, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 17, 17, 18, 18, 19,
    19, 20, 21, 21, 22, 23, 23, 24, 24, 25, 26, 26, 27, 28, 29, 29, 30, 31, 31, 32, 33, 34, 34, 35,
    36, 37, 37, 38, 39, 40, 40, 41, 42, 43, 44, 45, 45, 46, 47, 48, 49, 50, 51, 51, 52, 53, 54, 55,
    56, 57, 58, 59, 60, 61, 62, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 79,
    80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 92, 93, 94, 95, 96, 97, 98, 99, 101, 102, 103, 104,
    105, 107, 108, 109, 110, 111, 113, 114, 115, 116, 117, 119, 120, 121, 122, 124, 125, 126, 127,
    129, 130, 131, 133, 134, 135, 137, 138, 139, 141, 142, 143, 145, 146, 147, 149, 150, 151, 153,
    154, 156, 157, 158, 160, 161, 163, 164, 165, 167, 168, 170, 171, 173, 174, 175, 177, 178, 180,
    181, 183, 184, 186, 187, 189, 190, 192, 193, 195, 196, 198, 199, 201, 203, 204, 206, 207, 209,
    210, 212, 213, 215, 217, 218, 220, 221, 223, 225, 226, 228, 230, 231, 233, 234, 236, 238, 239,
    241, 243, 244, 246, 248, 249, 251, 253, 255,
];

const GAMMA_30: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3,
    4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10, 11, 11, 11,
    12, 12, 13, 13, 14, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 21, 21, 22, 22, 23, 24,
    24, 25, 25, 26, 27, 27, 28, 29, 30, 30, 31, 32, 33, 33, 34, 35, 36, 37, 37, 38, 39, 40, 41, 42,
    43, 44, 44, 45, 46, 47, 48, 49, 50, 51, 52, 54, 55, 56, 57, 58, 59, 60, 61, 62, 64, 65, 66, 67,
    69, 70, 71, 72, 74, 75, 76, 78, 79, 81, 82, 83, 85, 86, 88, 89, 91, 92, 94, 95, 97, 98, 100,
    102, 103, 105, 107, 108, 110, 112, 114, 115, 117, 119, 121, 123, 124, 126, 128, 130, 132, 134,
    136, 138, 140, 142, 144, 146, 148, 150, 152, 154, 157, 159, 161, 163, 165, 168, 170, 172, 175,
    177, 179, 182, 184, 187, 189, 192, 194, 197, 199, 202, 204, 207, 209, 212, 215, 217, 220, 223,
    226, 228, 231, 234, 237, 240, 243, 246, 249, 252, 255,
];

const GAMMA_22: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2,
    2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9, 10, 10, 10,
    11, 11, 12, 12, 13, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 21, 21, 22, 22,
    23, 23, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31, 31, 32, 33, 33, 34, 35, 36, 36, 37, 38, 39,
    40, 40, 41, 42, 43, 44, 45, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 55, 56, 57, 58, 59, 60,
    61, 62, 63, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 77, 78, 79, 80, 81, 82, 84, 85, 86, 87,
    88, 90, 91, 92, 93, 95, 96, 97, 99, 100, 101, 103, 104, 105, 107, 108, 109, 111, 112, 114, 115,
    117, 118, 119, 121, 122, 124, 125, 127, 128, 130, 131, 133, 135, 136, 138, 139, 141, 142, 144,
    146, 147, 149, 151, 152, 154, 156, 157, 159, 161, 162, 164, 166, 168, 169, 171, 173, 175, 176,
    178, 180, 182, 184, 186, 187, 189, 191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 213,
    215, 217, 219, 221, 223, 225, 227, 229, 231, 233, 235, 237, 239, 241, 244, 246, 248, 250, 252,
    255,
];

fn blue_to_dgreen(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 1024 to adjust the primary color to red.
    let dgreen_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 38) % (6 * 256),
        s: hsla.s,
        l: GAMMA_17[hsla.l as usize],
        a: hsla.a,
    };
    dgreen_adjust.into()
}

fn blue_to_ddgreen(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 1024 to adjust the primary color to red.
    let ddgreen_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 38) % (6 * 256),
        s: GAMMA_30[hsla.s as usize],
        l: GAMMA_22[hsla.l as usize],
        a: hsla.a,
    };
    ddgreen_adjust.into()
}

fn blue_to_dblue(input: &egui::Color32) -> egui::Color32 {
    let hsla = Hsla::from(input);
    // -324 adjusts the original blue green shirt to a primary color
    // 6 * 256 so the -324 won't cause the unsigned to go negative and panic the main thread
    // 1024 to adjust the primary color to red.
    let dblue_adjust = Hsla {
        h: (hsla.h + 6 * 256 - 324 + 350) % (6 * 256),
        s: GAMMA_30[hsla.s as usize],
        l: GAMMA_17[hsla.l as usize],
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
        l: GAMMA_17[hsla.l as usize],
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
    let mut num_bad_pixels = 0;
    let num_pixels: u32 = img.len().try_into().unwrap();

    for val in img.iter() {
        if val.a() != 0 && val.a() != 255 {
            num_bad_pixels += 1;
        }
    }
    let raw_percent = 100 * num_bad_pixels / num_pixels;
    if raw_percent == 0 && num_bad_pixels != 0 {
        1
    } else {
        raw_percent
    }
}

//fn compute_percent_opaque(img: &Vec<egui::Color32>) -> u32 {
fn compute_percent_opaque(img: &[egui::Color32]) -> u32 {
    let mut num_opaque_pixels = 0;
    let num_pixels: u32 = img.len().try_into().unwrap();

    for val in img.iter() {
        if val.a() > 0 {
            num_opaque_pixels += 1;
        }
    }
    100 * num_opaque_pixels / num_pixels
}

fn load_image_from_trusted_source(
    bytes: &[u8],
    name: impl Into<String>,
    ctx: &egui::Context,
) -> LoadedImage {
    let uncompressed_image = Arc::new(egui_extras::image::load_image_bytes(bytes).unwrap());
    let handle: egui::TextureHandle =
        ctx.load_texture(name, uncompressed_image.clone(), Default::default());
    LoadedImage {
        uncompressed_image,
        texture: handle,
    }
}

fn load_image_from_existing_image(
    existing: &LoadedImage,
    mutator: fn(&egui::Color32) -> egui::Color32,
    name: impl Into<String>,
    ctx: &egui::Context,
) -> LoadedImage {
    let mut new_image = Vec::with_capacity(existing.pixels().len());

    let in_pixels = existing.pixels();
    new_image.extend(in_pixels.iter().map(mutator));

    let uncompressed_image = Arc::new(egui::ColorImage {
        size: *(existing.size_as_array()),
        pixels: new_image,
    });
    let handle: egui::TextureHandle =
        ctx.load_texture(name, uncompressed_image.clone(), Default::default());
    LoadedImage {
        uncompressed_image,
        texture: handle,
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

pub struct ArtworkDependentData {
    bad_tpixel_percent: u32,
    opaque_percent: u32,
    fixed_artwork: LoadedImage,
    flagged_artwork: LoadedImage,
}

impl ArtworkDependentData {
    //fn new( cc: &eframe::CreationContext<'_>,
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
        Self {
            bad_tpixel_percent: compute_bad_tpixels(artwork.pixels()),
            opaque_percent: compute_percent_opaque(artwork.pixels()),
            fixed_artwork: default_fixed_art,
            flagged_artwork: default_flagged_art,
        }
    }
}

pub struct ReportTemplate<'a> {
    label: &'a str,
    display_percent: bool,
    metric_to_status: fn(metric: u32) -> ReportStatus,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp<'a> {
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
    t_shirt: egui::TextureId,
    zoom: f32,
    target: Vector3<f32>,
    last_drag_pos: std::option::Option<Vector3<f32>>,
    drag_display_to_tshirt: std::option::Option<Matrix3<f32>>,
    drag_count: i32,
    start_time: SystemTime,
    area_used_report: ReportTemplate<'a>,
    transparency_report: ReportTemplate<'a>,
    opaque_report: ReportTemplate<'a>,
    dpi_report: ReportTemplate<'a>,
    tool_selected_for: std::option::Option<ReportTypes>,
    tshirt_selected_for: TShirtColors,
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

fn v3_to_egui(item: Vector3<f32>) -> egui::Pos2 {
    egui::Pos2 {
        x: item.x,
        y: item.y,
    }
}

fn _eguip_to_v3(item: egui::Pos2) -> Vector3<f32> {
    vector![item.x, item.y, 1.0]
}

fn _eguiv_to_v3(item: egui::Vec2) -> Vector3<f32> {
    vector![item[0], item[1], 1.0]
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
    fn is_tool_active(&self, report_type: ReportTypes) -> bool {
        self.tool_selected_for.is_some() && self.tool_selected_for.unwrap() == report_type
    }

    fn do_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
            if DEBUG {
                ui.horizontal(|ui| unsafe {
                    ui.label("Bytes in file: ");
                    let copy = HELLO.clone();
                    ui.label(&copy);
                });
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

            let art_space_to_display =
                tshirt_to_display * self.art_space_to_tshirt();
            let art_to_display = art_space_to_display * self.art_to_art_space();

            let a0 = v3_to_egui(art_to_display * dvector![0.0, 0.0, 1.0]);
            let a1 = v3_to_egui(art_to_display * dvector![1.0, 1.0, 1.0]);

            if let Some(pointer_pos) = response.interact_pointer_pos() {
                let current_drag_pos = vector!(pointer_pos[0], pointer_pos[1], 1.0);

                if let Some(last_drag_pos) = self.last_drag_pos {
                    let display_to_artspace = self.drag_display_to_tshirt.unwrap();
                    let last = display_to_artspace * last_drag_pos;
                    let curr = display_to_artspace * current_drag_pos;
                    self.target = self.target + last - curr;
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
            } else {
                self.get_selected_art().id()
            };

            painter.image(
                texture_to_display,
                egui::Rect::from_min_max(a0, a1),
                egui::Rect::from_min_max(uv0, uv1),
                egui::Color32::WHITE,
            );

            if self.is_tool_active(ReportTypes::AreaUsed) {
                let art_space_border = vec![
                    v3_to_egui(art_space_to_display * dvector![0.0, 0.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![11.0, 0.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![11.0, 14.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![0.0, 14.0, 1.0]),
                    v3_to_egui(art_space_to_display * dvector![0.0, 0.0, 1.0]),
                ];

                let dash_dim = (art_space_to_display * dvector![0.2, 0.05, 1.0]) - (art_space_to_display * dvector![0.0, 0.0, 1.0]);
                let dash_length = dash_dim.x;
                let dash_width  = dash_dim.y;
                let gap_length  = dash_length;

                // animate with 3 cycles
                let cycle = (time_in_ms % (TRANSPARENCY_TOGGLE_RATE*3))/TRANSPARENCY_TOGGLE_RATE;
                let offset : f32 = (cycle as f32) / 3.0 * (dash_length + gap_length);
                let stroke_1 = egui::Stroke::new(dash_width, egui::Color32::from_rgb(200, 200, 200));

                painter.add(egui::Shape::dashed_line_with_offset( 
                    &art_space_border, stroke_1, &[dash_length], &[gap_length], offset));
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
            0..=74 => ReportStatus::Fail,
            75..=149 => ReportStatus::Warn,
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

    fn report_type_to_template(&self, report_type: ReportTypes) -> &ReportTemplate<'_> {
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

                    strip.cell(|ui| {
                        ui.add(status_icon);
                    });
                    strip.cell(|ui| {
                        ui.label(mtexts(&report.label.to_string()));
                    });
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(mtexts(&format!("{}", metric)));
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
                                            .max_width(TOOL_WIDTH),
                                    )
                                    .selected(is_selected),
                                )
                                .clicked()
                            {
                                self.tool_selected_for =
                                    if is_selected { None } else { Some(report_type) };
                            }
                        }
                    });
                });
        });
    }

    fn do_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("stuff")
            .resizable(true)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    let panel_size = ui.available_size_before_wrap();
                    self.footer_debug_0 = format!("{} {}", panel_size[0], panel_size[1]);
                    ui.add_space(10.0);
                    ui.vertical_centered(|ui| {
                        ui.heading(egui::widget_text::RichText::from("T-Shirt Checker").size(40.0))
                    });
                    ui.add_space(10.0);

                    self.report_metric(ui, ReportTypes::Dpi, self.compute_dpi());
                    self.report_metric(ui, ReportTypes::AreaUsed, self.compute_area_used());
                    self.report_metric(
                        ui,
                        ReportTypes::BadTransparency,
                        self.compute_badtransparency_pixels(),
                    );
                    self.report_metric(
                        ui,
                        ReportTypes::Opaqueness,
                        self.compute_opaque_percentage(),
                    );

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
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
                    ui.horizontal(|ui| {
                        self.handle_art_button(ctx, ui, Artwork::Artwork0);
                        self.handle_art_button(ctx, ui, Artwork::Artwork1);
                        self.handle_art_button(ctx, ui, Artwork::Artwork2);
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

        let dpi_report = ReportTemplate {
            label: "DPI",
            display_percent: false,
            metric_to_status: TShirtCheckerApp::dpi_to_status,
        };
        let area_used_report = ReportTemplate {
            label: "Area Used",
            display_percent: true,
            metric_to_status: TShirtCheckerApp::area_used_to_status,
        };
        let transparency_report = ReportTemplate {
            label: "Bad T.Pixels",
            display_percent: true,
            metric_to_status: TShirtCheckerApp::bad_transparency_to_status,
        };
        let opaque_report = ReportTemplate {
            label: "Bib Score",
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
        }
    }
}

fn mtexts(text: &String) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

impl eframe::App for TShirtCheckerApp<'_> {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.footer_debug_1 = format!("time {}", self.start_time.elapsed().unwrap().as_millis());
        self.do_bottom_panel(ctx);
        self.do_right_panel(ctx);
        self.do_central_panel(ctx);
        if self.is_tool_active(ReportTypes::BadTransparency) || self.is_tool_active(ReportTypes::AreaUsed) {
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
