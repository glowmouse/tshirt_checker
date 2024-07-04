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
    let new_a = if input.a() == 0 { 0 } else { 255 };
    egui::Color32::from_rgba_premultiplied(input.r(), input.g(), input.b(), new_a)
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

