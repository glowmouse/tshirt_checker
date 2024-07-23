use crate::image_utils::*;
use crate::loaded_image::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TShirtColors {
    Red,
    DRed,
    Green,
    DGreen,
    Blue,
    DBlue,
}

pub struct TShirtStorage {
    blue_t_shirt: LoadedImage,
    red_t_shirt: LoadedImage,
    dgreen_t_shirt: LoadedImage,
    burg_t_shirt: LoadedImage,
    dblue_t_shirt: LoadedImage,
    ddgreen_t_shirt: LoadedImage,
}

impl TShirtStorage {
    pub fn new(ctx: &egui::Context) -> Self {
        let blue_shirt: LoadedImage = load_image_from_trusted_source(
            include_bytes!("../assets/blue_tshirt.png"),
            "blue_shirt",
            ctx,
        );
        let red_mutator = blue_to_red();
        let red_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, &red_mutator, "red_shirt", ctx);
        let dgreen_mutator = blue_to_dgreen();
        let dgreen_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, &dgreen_mutator, "dgreen_shirt", ctx);
        let ddgreen_mutator = blue_to_ddgreen();
        let ddgreen_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, &ddgreen_mutator, "ddgreen_shirt", ctx);
        let dblue_mutator = blue_to_dblue();
        let dblue_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, &dblue_mutator, "dblue_shirt", ctx);

        let burg_mutator = blue_to_burg();
        let burg_shirt: LoadedImage =
            load_image_from_existing_image(&blue_shirt, &burg_mutator, "burg_shirt", ctx);

        Self {
            blue_t_shirt: blue_shirt,
            red_t_shirt: red_shirt,
            dgreen_t_shirt: dgreen_shirt,
            burg_t_shirt: burg_shirt,
            dblue_t_shirt: dblue_shirt,
            ddgreen_t_shirt: ddgreen_shirt,
        }
    }

    pub fn tshirt_enum_to_image(&self, color: TShirtColors) -> &LoadedImage {
        match color {
            TShirtColors::Red => &self.red_t_shirt,
            TShirtColors::DRed => &self.burg_t_shirt,
            TShirtColors::Green => &self.dgreen_t_shirt,
            TShirtColors::DGreen => &self.ddgreen_t_shirt,
            TShirtColors::Blue => &self.blue_t_shirt,
            TShirtColors::DBlue => &self.dblue_t_shirt,
        }
    }

    pub fn size(&self) -> egui::Vec2 {
        self.blue_t_shirt.size() // any shirt will do.
    }
}
