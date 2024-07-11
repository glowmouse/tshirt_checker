use crate::loaded_image::*;
use crate::Hsla;
use std::cmp::Ordering;

pub fn blue_to_red(input: &egui::Color32) -> egui::Color32 {
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

pub fn blue_to_dgreen(input: &egui::Color32) -> egui::Color32 {
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

pub fn blue_to_ddgreen(input: &egui::Color32) -> egui::Color32 {
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

pub fn blue_to_dblue(input: &egui::Color32) -> egui::Color32 {
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

pub fn blue_to_burg(input: &egui::Color32) -> egui::Color32 {
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

pub fn correct_alpha_for_tshirt(input: &egui::Color32) -> egui::Color32 {
    if input.a() < 255 {
        egui::Color32::TRANSPARENT
    } else {
        egui::Color32::from_rgb(input.r(), input.g(), input.b())
    }
}

pub fn flag_alpha_for_shirt(input: &egui::Color32) -> egui::Color32 {
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

pub fn compute_bad_tpixels(img: &[egui::Color32]) -> u32 {
    let num_pixels: u32 = img.len().try_into().unwrap();
    let num_bad_pixels: u32 = img
        .iter()
        .filter(|&p| p.a() != 0 && p.a() != 255)
        .count()
        .try_into()
        .unwrap();
    (100 * num_bad_pixels).div_ceil(num_pixels)
}

pub fn compute_percent_opaque(img: &[egui::Color32]) -> u32 {
    let num_pixels: u32 = img.len().try_into().unwrap();
    let num_opaque_pixels: u32 = img
        .iter()
        .filter(|&p| p.a() > 0)
        .count()
        .try_into()
        .unwrap();
    (100 * num_opaque_pixels).div_ceil(num_pixels)
}

#[derive(Eq, Clone)]
pub struct HotSpot {
    pub strength: u8,
    pub location: egui::Vec2,
}

impl PartialEq for HotSpot {
    fn eq(&self, other: &Self) -> bool {
        self.strength == other.strength && self.location == other.location
    }
}

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

pub fn hot_spots_from_heat_map(heat_map: &LoadedImage) -> Vec<HotSpot> {
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

struct ThinLineState<'a, const N: usize> {
    input: &'a Vec<egui::Color32>,
    output: &'a mut Vec<u32>,
    last_pixels: [usize; N],
    ring_index: usize,
    current_pixels: usize,
    max_pixels: usize,
}

impl<'a, const N: usize> ThinLineState<'a, N> {
    fn new(input: &'a Vec<egui::Color32>, output: &'a mut Vec<u32>, max_pixels: usize) -> Self {
        Self {
            input,
            output,
            last_pixels: [0; N],
            current_pixels: 0,
            ring_index: 0,
            max_pixels,
        }
    }
    #[inline(always)]
    pub fn transparent(&mut self) {
        if self.current_pixels > 0 && self.current_pixels <= self.max_pixels {
            for c in 0..self.current_pixels {
                let index = self.last_pixels[(N - c + self.ring_index) % N];
                self.output[index] += 1
            }
        }
        self.current_pixels = 0;
    }

    #[inline(always)]
    fn opaque(&mut self) {
        self.current_pixels += 1;
    }

    #[inline(always)]
    fn pixel(&mut self, index: usize) {
        let transparent = self.input[index].a() == 0;
        if !transparent {
            self.opaque();
        }
        if transparent {
            self.transparent();
        }
        self.ring_index = (self.ring_index + 1) % N;
        self.last_pixels[self.ring_index] = index;
    }
}

const RBS: usize = 32;

fn thin_line_vertical(
    output: &mut Vec<u32>,
    input: &Vec<egui::Color32>,
    xdim: usize,
    ydim: usize,
    min_pixels: usize,
) {
    let mut thin_line_state: ThinLineState<'_, RBS> = ThinLineState::new(input, output, min_pixels);

    for x in 0..xdim {
        for y in 0..ydim {
            thin_line_state.pixel(x + y * xdim);
        }
        thin_line_state.transparent();
    }
}

fn thin_line_horizontal(
    output: &mut Vec<u32>,
    input: &Vec<egui::Color32>,
    xdim: usize,
    ydim: usize,
    min_pixels: usize,
) {
    let mut thin_line_state: ThinLineState<'_, RBS> = ThinLineState::new(input, output, min_pixels);
    for y in 0..ydim {
        for x in 0..xdim {
            thin_line_state.pixel(x + y * xdim);
        }
        thin_line_state.transparent();
    }
}

#[allow(clippy::too_many_arguments)]
fn thin_line_diag_vdominant_execute(
    output: &mut Vec<u32>,
    input: &Vec<egui::Color32>,
    xdim: usize,
    ydim: usize,
    min_pixels: usize,
    vdelta: u32,
    xin: usize,
    yin: usize,
) {
    let mut x = xin;
    let mut y = yin;
    let mut thin_line_state: ThinLineState<'_, RBS> = ThinLineState::new(input, output, min_pixels);
    let mut fraction = vdelta / 2;

    while x != xdim && y != ydim {
        thin_line_state.pixel(x + y * xdim);
        y += 1;
        fraction += vdelta;
        if fraction >= 256 {
            fraction -= 256;
            x += 1;
        }
    }
}

fn thin_line_diag_vdominant(
    output: &mut Vec<u32>,
    input: &Vec<egui::Color32>,
    xdim: usize,
    ydim: usize,
    min_pixels: usize,
    vdelta: u32,
) {
    for y in 0..ydim {
        thin_line_diag_vdominant_execute(output, input, xdim, ydim, min_pixels, vdelta, 0, y);
    }
    for x in 0..xdim {
        thin_line_diag_vdominant_execute(output, input, xdim, ydim, min_pixels, vdelta, x, 0);
    }
}

#[allow(clippy::too_many_arguments)]
fn thin_line_diag_hdominant_execute(
    output: &mut Vec<u32>,
    input: &Vec<egui::Color32>,
    xdim: usize,
    ydim: usize,
    min_pixels: usize,
    vdelta: u32,
    xin: usize,
    yin: usize,
) {
    let mut x = xin;
    let mut y = yin;
    let mut thin_line_state: ThinLineState<'_, RBS> = ThinLineState::new(input, output, min_pixels);
    let mut fraction = vdelta / 2;

    while x != xdim && y != ydim {
        thin_line_state.pixel(x + y * xdim);
        x += 1;
        fraction += vdelta;
        if fraction >= 256 {
            fraction -= 256;
            y += 1;
        }
    }
}

fn thin_line_diag_hdominant(
    output: &mut Vec<u32>,
    input: &Vec<egui::Color32>,
    xdim: usize,
    ydim: usize,
    min_pixels: usize,
    vdelta: u32,
) {
    for y in 0..ydim {
        thin_line_diag_hdominant_execute(output, input, xdim, ydim, min_pixels, vdelta, 0, y);
    }
    for x in 0..xdim {
        thin_line_diag_hdominant_execute(output, input, xdim, ydim, min_pixels, vdelta, x, 0);
    }
}

fn thin_line_detect(input: &Vec<egui::Color32>, size: [usize; 2]) -> Vec<egui::Color32> {
    let xdim = size[0];
    let ydim = size[1];
    let min_pixels = 4;

    let mut output: Vec<u32> = vec![0; input.len()];

    thin_line_vertical(&mut output, input, xdim, ydim, min_pixels);
    thin_line_horizontal(&mut output, input, xdim, ydim, min_pixels);
    thin_line_diag_hdominant(&mut output, input, xdim, ydim, min_pixels, 256);
    thin_line_diag_hdominant(&mut output, input, xdim, ydim, min_pixels, 128);
    thin_line_diag_vdominant(&mut output, input, xdim, ydim, min_pixels, 128);

    input
        .iter()
        .zip(output)
        .map(|(a, b)| {
            if b < 3 {
                *a
            } else {
                egui::Color32::from_rgb(255 - a.r(), 255 - a.g(), 255 - a.b())
            }
        })
        .collect()
}

pub fn flag_thin_lines(input: &LoadedImage, ctx: &egui::Context) -> LoadedImage {
    let output = thin_line_detect(input.pixels(), *input.size_as_array());
    return load_image_from_pixels(output, *input.size_as_array(), "thin_lines", ctx);
}

pub fn count_diffs(in0: &LoadedImage, in1: &LoadedImage) -> usize {
    let in0_pixels = in0.pixels();
    let in1_pixels = in1.pixels();

    in0_pixels
        .iter()
        .zip(in1_pixels)
        .filter(|&(a, b)| a == b)
        .count()
}

pub fn compute_percent_diff(in0: &LoadedImage, in1: &LoadedImage) -> u32 {
    let thin_line_pixels: u32 = count_diffs(in0, in1).try_into().unwrap();
    let total_pixels: u32 = (in0.size()[0] * in0.size()[1]) as u32;
    let percent_thin_line: u32 = thin_line_pixels / total_pixels;
    if percent_thin_line == 0 && thin_line_pixels != 0 {
        1
    } else {
        percent_thin_line
    }
}
