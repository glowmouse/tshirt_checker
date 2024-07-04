use crate::image_utils::*;
use crate::loaded_image::*;

#[derive(PartialEq, Copy, Clone)]
pub enum Artwork {
    Artwork0,
    Artwork1,
    Artwork2,
}

pub struct ArtworkDependentData {
    pub partial_transparency_percent: u32,
    pub opaque_percent: u32,
    pub fixed_artwork: LoadedImage,
    pub flagged_artwork: LoadedImage,
    _heat_map: LoadedImage,
    pub top_hot_spots: Vec<HotSpot>,
}

impl ArtworkDependentData {
    pub fn new(ctx: &egui::Context, artwork: &LoadedImage) -> Self {
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
            partial_transparency_percent: compute_bad_tpixels(artwork.pixels()),
            opaque_percent: compute_percent_opaque(artwork.pixels()),
            fixed_artwork: default_fixed_art,
            flagged_artwork: default_flagged_art,
            top_hot_spots: hot_spots_from_heat_map(&heat_map),
            _heat_map: heat_map_from_image(artwork, "heatmap", ctx),
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
        let artwork_0: LoadedImage =
            load_image_from_trusted_source(include_bytes!("test_artwork.png"), "artwork_0", ctx);
        let artwork_1: LoadedImage = load_image_from_trusted_source(
            include_bytes!("sf2024-attendee-v1.png"),
            "artwork_1",
            ctx,
        );
        let artwork_2: LoadedImage = load_image_from_trusted_source(
            include_bytes!("sf2024-attendee-v2.png"),
            "artwork_2",
            ctx,
        );

        Self {
            art_dependent_data_0: Some(ArtworkDependentData::new(ctx, &artwork_0)),
            art_dependent_data_1: None,
            art_dependent_data_2: None,
            artwork_0,
            artwork_1,
            artwork_2,
        }
    }
    pub fn get_dependent_data(&self, artwork: Artwork) -> &ArtworkDependentData {
        // For now I guess I guarantee, through logic that's hard to reason about
        // that the unwrap always succeeds.  Definately a comments are a code
        // smell moment.
        match artwork {
            Artwork::Artwork0 => self.art_dependent_data_0.as_ref().unwrap(),
            Artwork::Artwork1 => self.art_dependent_data_1.as_ref().unwrap(),
            Artwork::Artwork2 => self.art_dependent_data_2.as_ref().unwrap(),
        }
    }

    pub fn cache_in_art_dependent_data(&mut self, ctx: &egui::Context, artwork: Artwork) {
        let image: &LoadedImage = self.get_art(artwork);

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
}
