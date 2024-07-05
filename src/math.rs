extern crate nalgebra as na;
use crate::loaded_image::*;
use na::{matrix, vector, Matrix3, Vector3};

#[derive(Debug)]
pub struct ViewPort {
    pub zoom: f32,
    pub target: Vector3<f32>,
    pub display_size: egui::Vec2,
    pub tshirt_size: egui::Vec2,
}

//
// Transforms from "tshirt space", where (0,0) is the top
// left corner of the tshirt image and (1,1) is the bottom
// right corner of the tshirt image, to the display.
//
pub fn tshirt_to_display(viewport: ViewPort) -> Matrix3<f32> {
    let display_aspect = viewport.display_size.x / viewport.display_size.y;
    let tshirt_aspect = viewport.tshirt_size.x / viewport.tshirt_size.y;

    let move_target_to_origin: Matrix3<f32> = matrix![ 1.0,  0.0,  -viewport.target.x;
                     0.0,  1.0,  -viewport.target.y;
                     0.0,  0.0,  1.0 ];
    let move_origin_to_center: Matrix3<f32> = matrix![ 1.0,  0.0,  0.5;
                     0.0,  1.0,  0.5;
                     0.0,  0.0,  1.0 ];
    let scale_at_origin: Matrix3<f32> = matrix![ viewport.zoom,  0.0,        0.0;
                     0.0,        viewport.zoom,  0.0;
                     0.0,        0.0,        1.0 ];

    let center_at_target_and_scale =
        move_origin_to_center * scale_at_origin * move_target_to_origin;

    let centered_tshirt_to_display = if display_aspect > tshirt_aspect {
        // Display is wider than the t-shirt

        // a. T-shirt occupies the entire Y dimension of the display
        // b. Adjust Y tshirt dimension by aspect ratio to get the X dimension
        // c. Divide the unused space in half and use it as the x margin
        let y_img_on_display_dim = viewport.display_size.y; // a
        let x_img_on_display_dim = y_img_on_display_dim * tshirt_aspect; // b
        let x_margin = (viewport.display_size.x - x_img_on_display_dim) / 2.0; // c
        matrix![  x_img_on_display_dim,    0.0,             x_margin;
                             0.0,        y_img_on_display_dim,   0.0;
                             0.0,        0.0,             1.0  ]
    } else {
        // display is higher than the t-shirt

        // a. T-shirt occupies the entire X dimension of the display
        // b. Adjust X tshirt dimension by aspect ratio to get the Y dimension
        // c. Divide the unused space in half and use it as the Y margin
        let x_img_on_display_dim = viewport.display_size.x;
        let y_img_on_display_dim = x_img_on_display_dim / tshirt_aspect;
        let y_margin = (viewport.display_size.y - y_img_on_display_dim) / 2.0;
        matrix![  x_img_on_display_dim,    0.0,             0.0;
                  0.0,              y_img_on_display_dim,         y_margin;
                  0.0,              0.0,             1.0  ]
    };
    centered_tshirt_to_display * center_at_target_and_scale
}

pub fn art_to_art_space(art: &LoadedImage) -> Matrix3<f32> {
    let artspace_size = vector!(11.0, 14.0);
    let artspace_aspect = artspace_size.x / artspace_size.y;

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
    // display is higher than the t-shirt
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
pub fn art_space_to_tshirt(tshirt_size: egui::Vec2) -> Matrix3<f32> {
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

pub fn v3_to_egui(item: Vector3<f32>) -> egui::Pos2 {
    egui::Pos2 {
        x: item.x,
        y: item.y,
    }
}

#[cfg(test)]
mod display_to_tshirt_should {
    use super::*;

    #[test]
    fn work_for_wide_displays() {
        let viewport = ViewPort {
            zoom: 1.0,
            target: vector![0.5, 0.5, 1.0],
            display_size: egui::Vec2::new(1000.0, 1000.0),
            tshirt_size: egui::Vec2::new(300.0, 500.0),
        };
        let actual = tshirt_to_display(viewport);
        let expected = matrix![ 600.0, 0.0, 200.0 ;
                            0.0,   1000.0, 0.0 ;
                            0.0,   0.0,    1.0 ];
        assert_eq!(actual, expected);
    }
    #[test]
    fn work_for_non_centered_wide_displays() {
        let viewport = ViewPort {
            zoom: 1.0,
            target: vector![1.0, 1.0, 1.0],
            display_size: egui::Vec2::new(1000.0, 1000.0),
            tshirt_size: egui::Vec2::new(300.0, 500.0),
        };
        let actual = tshirt_to_display(viewport);
        let expected = matrix![ 600.0, 0.0, -100.0 ;
                            0.0,   1000.0, -500.0 ;
                            0.0,   0.0,    1.0 ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn work_for_tall_displays() {
        let viewport = ViewPort {
            zoom: 1.0,
            target: vector![0.5, 0.5, 1.0],
            display_size: egui::Vec2::new(1000.0, 2000.0),
            tshirt_size: egui::Vec2::new(500.0, 500.0),
        };
        let actual = tshirt_to_display(viewport);
        let expected = matrix![ 1000.0, 0.0, 0.0 ;
                            0.0,   1000.0, 500.0 ;
                            0.0,   0.0,    1.0 ];
        assert_eq!(actual, expected);
    }

    #[test]
    fn worked_for_tall_displays_zoomed() {
        let viewport = ViewPort {
            zoom: 3.0,
            target: vector![0.5, 0.5, 1.0],
            display_size: egui::Vec2::new(1000.0, 2000.0),
            tshirt_size: egui::Vec2::new(500.0, 500.0),
        };
        let actual = tshirt_to_display(viewport);
        let expected = matrix![ 3000.0, 0.0, -1000.0 ;
                            0.0,   3000.0, -500.0 ;
                            0.0,   0.0,    1.0 ];
        assert_eq!(actual, expected);
    }
}
