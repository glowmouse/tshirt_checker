extern crate nalgebra as na;
use crate::loaded_image::*;
use na::{matrix, vector, Matrix3, Vector3};

pub struct ViewPort {
    pub zoom: f32,
    pub target: Vector3<f32>,
    pub panel_size: egui::Vec2,
    pub tshirt_size: egui::Vec2,
}

//
// Transforms from "t shirt space", where (0,0) is the top
// left corner of the t shirt image and (1,1) is the bottom
// right corner of the t-shirt image, to the display.
//
pub fn tshirt_to_display(viewport: ViewPort) -> Matrix3<f32> {
    let panel_aspect = viewport.panel_size[0] / viewport.panel_size[1];
    let tshirt_aspect = viewport.tshirt_size.x / viewport.tshirt_size.y;

    let move_from_center: Matrix3<f32> = matrix![ 1.0,  0.0,  -viewport.target.x;
                     0.0,  1.0,  -viewport.target.y;
                     0.0,  0.0,  1.0 ];
    let move_to_center: Matrix3<f32> = matrix![ 1.0,  0.0,  0.5;
                     0.0,  1.0,  0.5;
                     0.0,  0.0,  1.0 ];
    let scale: Matrix3<f32> = matrix![ viewport.zoom,  0.0,        0.0;
                     0.0,        viewport.zoom,  0.0;
                     0.0,        0.0,        1.0 ];

    let scale_centered = move_to_center * scale * move_from_center;

    if panel_aspect > tshirt_aspect {
        // panel is wider than the t-shirt
        let x_width = viewport.panel_size[0] * tshirt_aspect / panel_aspect;
        let x_margin = (viewport.panel_size[0] - x_width) / 2.0;
        return matrix![  x_width,    0.0,             x_margin;
                             0.0,        viewport.panel_size[1],   0.0;
                             0.0,        0.0,             1.0  ]
            * scale_centered;
    }
    // panel is higher than the t-shirt
    let y_width = viewport.panel_size[1] / tshirt_aspect * panel_aspect;
    let y_margin = (viewport.panel_size[1] - y_width) / 2.0;
    matrix![  viewport.panel_size[0],    0.0,             0.0;
                  0.0,              y_width,         y_margin;
                  0.0,              0.0,             1.0  ]
        * scale_centered
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
