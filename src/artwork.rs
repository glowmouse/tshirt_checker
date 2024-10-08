//! Tools for creating and managing artwork analysis data (dependant data)
//! given the t-shirt artwork as input.

use crate::image_utils::*;
use crate::loaded_image::*;
use crate::math::*;
use nalgebra::dvector;

/// Artwork slot - one of three.
#[derive(PartialEq, Copy, Clone)]
pub enum ArtEnum {
    Artwork0,
    Artwork1,
    Artwork2,
}

/// Analysis data that depends on the t-shirt artwork
pub struct ArtworkDependentData {
    // Data for DPI tool
    dpi_top_hot_spots: Vec<HotSpot>,

    // Data for Partial Transparency report/ tool
    partial_transparency_percent: u32,
    partial_transparency_problems: LoadedImage,
    partial_transparency_fixed: LoadedImage,

    // Data for Bib report/ tool
    bib_opaque_percent: u32,
    bib_opaque_mask: LoadedImage,

    // Data for Thin Line tool
    thin_line_percent: u32,
    thin_line_problems: LoadedImage,
}

impl ArtworkDependentData {
    // Compute data for reports and report tools asyncronously.
    //
    // My understanding is that the tranditional web server model is single
    // threaded.  Refreshes shouldn't block for long chunks of time.
    //
    // This code is a best effort to compute data t-shirt checker needs for reports
    // asyncronously.  When run as a native app on Linux this is actually concurrent.
    // When run in web assembly on a browser it's more of a co-operative multi-tasking
    // model.
    //
    // The context_switch().await calls let the web assembly build know this is a spot where
    // it can give something else a change to run
    //
    pub async fn new(ctx: &egui::Context, artwork: &LoadedImage) -> Self {
        //
        // Compute interesting hot spots for the DPI tool using a heat map based
        // on a simple edge detection algorithm,
        //
        crate::async_tasks::context_switch(ctx).await;
        let heat_map = heat_map_from_image(artwork, "heatmap", ctx);
        let dpi_top_hot_spots = hot_spots_from_heat_map(&heat_map);

        //
        // Create images and metrics for the partial transparency tool
        //
        crate::async_tasks::context_switch(ctx).await;
        let partial_transparency_problems: LoadedImage = load_image_from_existing_image(
            artwork,
            &flag_alpha_for_shirt,
            "partial_transparency_problems",
            ctx,
        );
        crate::async_tasks::context_switch(ctx).await;
        let partial_transparency_fixed: LoadedImage = load_image_from_existing_image(
            artwork,
            &correct_alpha_for_tshirt,
            "partial_transparency_fixed",
            ctx,
        );
        crate::async_tasks::context_switch(ctx).await;
        let partial_transparency_percent = compute_bad_tpixels(artwork.pixels());

        //
        // Compute images and metrics for the bib report.
        //
        crate::async_tasks::context_switch(ctx).await;
        let bib_opaque_percent = compute_percent_opaque(artwork.pixels());
        crate::async_tasks::context_switch(ctx).await;
        let bib_opaque_mask =
            load_image_from_existing_image(artwork, &opaque_to_mask, "bib_mask", ctx);

        //
        // Compute images and metrics for the thin line report & tool
        //
        crate::async_tasks::context_switch(ctx).await;
        let top_corner = art_to_art_space(artwork.size()) * dvector![0.0, 0.0, 1.0];
        let bot_corner = art_to_art_space(artwork.size()) * dvector![1.0, 1.0, 1.0];
        let dim_in_inches = bot_corner - top_corner;
        let dpi = artwork.size().x / dim_in_inches.x;
        // going to say that 1/64 inches is too thin for now
        let dots = (dpi * (1.0 / 64.0)).ceil() as usize;

        let thin_line_problems = flag_thin_lines(artwork, ctx, dots).await;
        crate::async_tasks::context_switch(ctx).await;
        let thin_line_percent = compute_percent_diff(&thin_line_problems, artwork);

        Self {
            dpi_top_hot_spots,

            partial_transparency_percent,
            partial_transparency_problems,
            partial_transparency_fixed,

            bib_opaque_percent,
            bib_opaque_mask,

            thin_line_percent,
            thin_line_problems,
        }
    }

    pub fn dpi_top_hot_spots(&self) -> &Vec<HotSpot> {
        &self.dpi_top_hot_spots
    }

    pub fn partial_transparency_percent(&self) -> u32 {
        self.partial_transparency_percent
    }

    pub fn partial_transparency_problems(&self) -> &LoadedImage {
        &self.partial_transparency_problems
    }

    pub fn partial_transparency_fixed(&self) -> &LoadedImage {
        &self.partial_transparency_fixed
    }

    pub fn bib_opaque_percent(&self) -> u32 {
        self.bib_opaque_percent
    }

    pub fn bib_opaque_mask(&self) -> &LoadedImage {
        &self.bib_opaque_mask
    }

    pub fn thin_line_percent(&self) -> u32 {
        self.thin_line_percent
    }

    pub fn thin_line_problems(&self) -> &LoadedImage {
        &self.thin_line_problems
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
    pub fn get_dependent_data(&self, art_id: ArtEnum) -> Option<&ArtworkDependentData> {
        match art_id {
            ArtEnum::Artwork0 => self.art_dependent_data_0.as_ref(),
            ArtEnum::Artwork1 => self.art_dependent_data_1.as_ref(),
            ArtEnum::Artwork2 => self.art_dependent_data_2.as_ref(),
        }
    }

    pub fn get_art(&self, art_id: ArtEnum) -> &LoadedImage {
        match art_id {
            ArtEnum::Artwork0 => &self.artwork_0,
            ArtEnum::Artwork1 => &self.artwork_1,
            ArtEnum::Artwork2 => &self.artwork_2,
        }
    }

    pub fn set_art(
        &mut self,
        art_id: ArtEnum,
        image: LoadedImage,
        dependent_data: Option<ArtworkDependentData>,
    ) {
        match art_id {
            ArtEnum::Artwork0 => {
                self.artwork_0 = image;
                self.art_dependent_data_0 = dependent_data;
            }
            ArtEnum::Artwork1 => {
                self.artwork_1 = image;
                self.art_dependent_data_1 = dependent_data;
            }
            ArtEnum::Artwork2 => {
                self.artwork_2 = image;
                self.art_dependent_data_2 = dependent_data;
            }
        }
    }
}
