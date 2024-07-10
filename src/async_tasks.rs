use crate::artwork::*;
use crate::loaded_image::*;

pub struct ImageLoad {
    pub artwork: Artwork,
    pub image: LoadedImage,
    pub dependent_data: ArtworkDependentData,
}

//
// Copied from https://github.com/PolyMeilex/rfd/blob/master/examples/async.rs
//
// My current understanding (new to this) is that nothing executed in web
// assembly can block the main thread...  and the thread mechanism used by
// web assembly won't return the thread's output.
//

use std::future::Future;

#[cfg(not(target_arch = "wasm32"))]
fn app_execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    // this is stupid... use any executor of your choice instead
    std::thread::spawn(move || async_std::task::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn app_execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

pub fn do_load(
    ctx: &egui::Context,
    art_slot: Artwork,
    sender: &std::sync::mpsc::Sender<Result<ImageLoad, String>>,
) {
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    // Execute in another thread
    app_execute(async move {
        let file = rfd::AsyncFileDialog::new().pick_file().await;
        let data: Vec<u8> = file.unwrap().read().await;

        let image = load_image_from_untrusted_source(&data, "loaded_data", &thread_ctx).unwrap();
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &image).await;

        let send_image = Ok(ImageLoad {
            artwork: art_slot,
            image,
            dependent_data,
        });

        thread_sender.send(send_image).unwrap();
        thread_ctx.request_repaint();
    });
}

pub fn partialt_fix(
    ctx: &egui::Context,
    art: &LoadedImage,
    art_id: Artwork,
    sender: &std::sync::mpsc::Sender<Result<ImageLoad, String>>,
) {
    // Execute in another thread
    let thread_art = art.clone();
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    app_execute(async move {
        let fixed_art = load_image_from_existing_image(
            &thread_art,
            |p| {
                let new_alpha: u8 = if p.a() < 25 { 0 } else { 255 };
                egui::Color32::from_rgba_premultiplied(p.r(), p.g(), p.b(), new_alpha)
            },
            "fixed_art", // todo, better name...
            &thread_ctx,
        );
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &fixed_art).await;
        let image_to_send = Ok(ImageLoad {
            artwork: art_id,
            image: fixed_art,
            dependent_data,
        });
        thread_sender.send(image_to_send).unwrap();
        thread_ctx.request_repaint();
    });
}

pub fn cache_in_dependent_data(
    ctx: &egui::Context,
    art: &LoadedImage,
    art_id: Artwork,
    sender: &std::sync::mpsc::Sender<Result<ImageLoad, String>>,
) {
    let thread_art = art.clone();
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    app_execute(async move {
        async_std::task::yield_now().await;
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &thread_art).await;
        async_std::task::yield_now().await;
        let image_to_send = Ok(ImageLoad {
            artwork: art_id,
            image: thread_art,
            dependent_data,
        });
        thread_sender.send(image_to_send).unwrap();
        thread_ctx.request_repaint();
    });
}