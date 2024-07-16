use crate::image_utils::*;
use crate::loaded_image::*;
use crate::math::*;
use nalgebra::dvector;

#[derive(PartialEq, Copy, Clone)]
pub enum Artwork {
    Artwork0,
    Artwork1,
    Artwork2,
}

pub struct ArtworkDependentData {
    pub partial_transparency_percent: u32,
    pub opaque_percent: u32,
    pub thin_line_percent: u32,
    pub fixed_artwork: LoadedImage,
    pub flagged_artwork: LoadedImage,
    pub thin_lines: LoadedImage,
    pub top_hot_spots: Vec<HotSpot>,
}

impl ArtworkDependentData {
    pub async fn new(ctx: &egui::Context, artwork: &LoadedImage) -> Self {
        let one_milli = std::time::Duration::from_millis(1);
        async_std::task::sleep(one_milli).await;
        let default_fixed_art: LoadedImage = load_image_from_existing_image(
            artwork,
            correct_alpha_for_tshirt,
            "fixed default art",
            ctx,
        );
        async_std::task::sleep(one_milli).await;
        let default_flagged_art: LoadedImage = load_image_from_existing_image(
            artwork,
            flag_alpha_for_shirt,
            "flagged default art",
            ctx,
        );
        async_std::task::sleep(one_milli).await;
        let heat_map = heat_map_from_image(artwork, "heatmap", ctx);
        async_std::task::sleep(one_milli).await;
        let opaque_percent = compute_percent_opaque(artwork.pixels());
        async_std::task::sleep(one_milli).await;
        let partial_transparency_percent = compute_bad_tpixels(artwork.pixels());
        async_std::task::sleep(one_milli).await;
        let top_hot_spots = hot_spots_from_heat_map(&heat_map);
        async_std::task::sleep(one_milli).await;

        // Cut and paste from report_templates.rs, should refactor.
        let top_corner = art_to_art_space(artwork.size()) * dvector![0.0, 0.0, 1.0];
        let bot_corner = art_to_art_space(artwork.size()) * dvector![1.0, 1.0, 1.0];
        let dim_in_inches = bot_corner - top_corner;
        let dpi = artwork.size().x / dim_in_inches.x;
        // going to say that 1/64 inches is too thin for now
        let dots = (dpi * (1.0 / 64.0)).ceil() as usize;

        let thin_lines = flag_thin_lines(artwork, ctx, dots).await;
        async_std::task::sleep(one_milli).await;
        let thin_line_percent = compute_percent_diff(&thin_lines, artwork);
        async_std::task::sleep(one_milli).await;

        Self {
            partial_transparency_percent,
            opaque_percent,
            fixed_artwork: default_fixed_art,
            flagged_artwork: default_flagged_art,
            top_hot_spots,
            thin_lines,
            thin_line_percent,
        }
    }
}

pub struct ArtStorage {
    artwork_0: LoadedImage,
    artwork_1: LoadedImage,
    artwork_2: LoadedImage,
    art_dependent_data_0: std::option::Option<ArtworkDependentData>,
    art_dependent_data_1: std::option::Option<ArtworkDependentData>,
    art_dependent_data_2: std::option::Option<ArtworkDependentData>,
}

impl ArtStorage {
    pub fn new(ctx: &egui::Context) -> Self {
        let artwork_0: LoadedImage = load_image_from_trusted_source(
            include_bytes!("../assets/test_artwork.png"),
            "artwork_0",
            ctx,
        );
        let artwork_1: LoadedImage =
            load_image_from_trusted_source(include_bytes!("../assets/tux.svg"), "artwork_1", ctx);
        let artwork_2: LoadedImage = load_image_from_trusted_source(
            include_bytes!("../assets/rust_crab.svg"),
            "artwork_2",
            ctx,
        );

        Self {
            art_dependent_data_0: None,
            art_dependent_data_1: None,
            art_dependent_data_2: None,
            artwork_0,
            artwork_1,
            artwork_2,
        }
    }
    pub fn get_dependent_data(&self, artwork: Artwork) -> Option<&ArtworkDependentData> {
        match artwork {
            Artwork::Artwork0 => self.art_dependent_data_0.as_ref(),
            Artwork::Artwork1 => self.art_dependent_data_1.as_ref(),
            Artwork::Artwork2 => self.art_dependent_data_2.as_ref(),
        }
    }

    pub fn get_art(&self, artwork: Artwork) -> &LoadedImage {
        match artwork {
            Artwork::Artwork0 => &self.artwork_0,
            Artwork::Artwork1 => &self.artwork_1,
            Artwork::Artwork2 => &self.artwork_2,
        }
    }

    pub fn set_art(
        &mut self,
        slot: Artwork,
        image: LoadedImage,
        dependent_data: Option<ArtworkDependentData>,
    ) {
        match slot {
            Artwork::Artwork0 => {
                self.artwork_0 = image;
                self.art_dependent_data_0 = dependent_data;
            }
            Artwork::Artwork1 => {
                self.artwork_1 = image;
                self.art_dependent_data_1 = dependent_data;
            }
            Artwork::Artwork2 => {
                self.artwork_2 = image;
                self.art_dependent_data_2 = dependent_data;
            }
        }
    }
}
