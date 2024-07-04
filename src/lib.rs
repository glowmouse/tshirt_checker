#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TShirtCheckerApp;
mod hsla;
pub use hsla::Hsla;
mod gamma_tables;
mod loaded_image;
pub use loaded_image::LoadedImage;
mod image_utils;
mod report_templates;
