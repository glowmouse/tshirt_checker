use crate::artwork::*;
use crate::error::*;
use crate::image_utils::*;
use crate::loaded_image::*;

pub type Payload = Result<ImageLoad, Error>;
pub type Sender = std::sync::mpsc::Sender<Payload>;
pub type Receiver = std::sync::mpsc::Receiver<Payload>;

pub struct ImageLoad {
    pub artwork: Artwork,
    pub image: LoadedImage,
    pub dependent_data: Option<ArtworkDependentData>,
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

pub async fn load_image(ctx: &egui::Context) -> Result<LoadedImage, Error> {
    let file = rfd::AsyncFileDialog::new().pick_file().await;
    if file.is_none() {
        return Err(Error::new(
            ErrorTypes::FileImportAborted,
            "Image Import cancelled by user",
        ));
    }
    let data: Vec<u8> = file.unwrap().read().await;
    let image = load_image_from_untrusted_source(&data, "loaded_data", ctx)?;
    Ok(image)
}

pub fn do_load(ctx: &egui::Context, art_slot: Artwork, sender: &Sender) {
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    // Execute in another thread
    app_execute(async move {
        let image_maybe = load_image(&thread_ctx).await;
        if image_maybe.is_err() {
            thread_sender.send(Err(image_maybe.err().unwrap())).unwrap();
            thread_ctx.request_repaint();
            return;
        }
        let image = image_maybe.unwrap();

        let send_image = Ok(ImageLoad {
            artwork: art_slot,
            image: image.clone(),
            dependent_data: None,
        });
        thread_sender.send(send_image).unwrap();
        thread_ctx.request_repaint();

        let dependent_data = ArtworkDependentData::new(&thread_ctx, &image).await;

        let send_image_and_dep_data = Ok(ImageLoad {
            artwork: art_slot,
            image,
            dependent_data: Some(dependent_data),
        });

        thread_sender.send(send_image_and_dep_data).unwrap();
        thread_ctx.request_repaint();
    });
}

pub fn partialt_fix(ctx: &egui::Context, art: &LoadedImage, art_id: Artwork, sender: &Sender) {
    // Execute in another thread
    let thread_art = art.clone();
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    app_execute(async move {
        let fixed_art = load_image_from_existing_image(
            &thread_art,
            correct_alpha_for_tshirt,
            "blah_blah_fixed_art", // todo, better name...
            &thread_ctx,
        );
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &fixed_art).await;
        let image_to_send = Ok(ImageLoad {
            artwork: art_id,
            image: fixed_art,
            dependent_data: Some(dependent_data),
        });
        thread_sender.send(image_to_send).unwrap();
        thread_ctx.request_repaint();
    });
}

pub fn cache_in_dependent_data(
    ctx: &egui::Context,
    art: &LoadedImage,
    art_id: Artwork,
    sender: &Sender,
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
            dependent_data: Some(dependent_data),
        });
        thread_sender.send(image_to_send).unwrap();
        thread_ctx.request_repaint();
    });
}
