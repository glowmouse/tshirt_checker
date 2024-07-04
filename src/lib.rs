#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::TShirtCheckerApp;
mod hsla;
pub use hsla::Hsla;
mod gamma_tables;
