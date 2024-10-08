#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TShirtCheckerApp;
mod hsla;
pub use hsla::Hsla;
mod loaded_image;
pub use loaded_image::LoadedImage;
mod image_utils;
mod report_templates;
mod tshirt_storage;
pub use tshirt_storage::TShirtStorage;
mod artwork;
mod async_tasks;
mod error;
mod icons;
mod log;
mod math;
mod movement_state;
mod notice_panel;
mod time;
mod tool_select;
