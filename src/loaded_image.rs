//! Abstraction for an image that's loaded into memory
//!
//! By default, egui usually doesn't seem to load images in immediate mode, and doesn't allow
//! access to the image pixel data, which seems like the right thing to do in most use cases.
//!
//! A lot of the images I'm using are smallish icons, they're baked into the wasm executable,
//! and I just don't care if they're loading in immediate mode.  I added 
//! load_image_from_trusted_source so I could initialize images from a trusted source - say
//! constant bytes in the wasm executable where I know image is a properly done .png file.
//!
//! I have a second use case where I need to modify existing images (i.e., change t-shirt
//! colors).  For the sake of convenience, I'm always storing the underlying egui::ColorImage
//! 

use std::sync::Arc;

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

pub fn load_image_from_untrusted_source(
    bytes: &[u8],
    name: impl Into<String>,
    ctx: &egui::Context,
) -> Result<LoadedImage, String> {
    let uncompressed_image = Arc::new(egui_extras::image::load_image_bytes(bytes)?);
    let texture: egui::TextureHandle =
        ctx.load_texture(name, uncompressed_image.clone(), Default::default());
    Ok(LoadedImage {
        uncompressed_image,
        texture,
    })
}

pub fn load_image_from_trusted_source(
    bytes: &[u8],
    name: impl Into<String>,
    ctx: &egui::Context,
) -> LoadedImage {
    load_image_from_untrusted_source(bytes, name, ctx).unwrap()
}

pub fn load_image_from_existing_image(
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

pub fn heat_map_from_image(
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

