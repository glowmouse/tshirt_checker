use crate::artwork::*;
use crate::error::*;
use crate::image_utils::*;
use crate::loaded_image::*;
use std::future::Future;

pub type Payload = Result<ImageLoad, Error>;
pub type Sender = std::sync::mpsc::Sender<Payload>;
pub type Receiver = std::sync::mpsc::Receiver<Payload>;

pub struct ImageLoad {
    pub art_id: ArtEnum,
    pub image: LoadedImage,
    pub dependent_data: Option<ArtworkDependentData>,
}

//
// In web-assembly asyncronous tasks get run in a co-operative multi-tasking kind of
// way.  In non web-assembly asycnronous tasks are just run in another thread
//
// Copied from https://github.com/PolyMeilex/rfd/blob/master/examples/async.rs
//
#[cfg(not(target_arch = "wasm32"))]
fn app_execute<F: Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || async_std::task::block_on(f));
}
#[cfg(target_arch = "wasm32")]
fn app_execute<F: Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

//
// Explicite context switch.  This also sends a repaint request to the main GUI so
// load animations get repainted.
//
pub async fn context_switch(ctx: &egui::Context) {
    ctx.request_repaint();
    let one_milli = std::time::Duration::from_millis(1);
    async_std::task::sleep(one_milli).await;
}

//
// Asyncronous file load using the rfd library
//
pub async fn load_image(ctx: &egui::Context) -> Result<LoadedImage, Error> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("All", &["png", "jpg", "jpeg", "jpe", "jif", "jtif", "svg"])
        .add_filter("Png Images", &["png"])
        .add_filter("Jpeg Images", &["jpg", "jpeg", "jpe", "jif", "jtif"])
        .add_filter("SVG Images", &["svg"])
        .pick_file()
        .await;

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

pub fn do_load(ctx: &egui::Context, art_id: ArtEnum, sender: &Sender) {
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    // Execute the load asyncronously so we don't block the main thread.
    //
    app_execute(async move {
        let image_maybe = load_image(&thread_ctx).await;
        if image_maybe.is_err() {
            thread_sender.send(Err(image_maybe.err().unwrap())).unwrap();
            thread_ctx.request_repaint();
            return;
        }
        let image = image_maybe.unwrap();

        context_switch(&thread_ctx).await;
        let send_image = Ok(ImageLoad {
            art_id,
            image: image.clone(),
            dependent_data: None,
        });
        thread_sender.send(send_image).unwrap();

        context_switch(&thread_ctx).await;
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &image).await;

        let send_image_and_dep_data = Ok(ImageLoad {
            art_id,
            image,
            dependent_data: Some(dependent_data),
        });

        thread_sender.send(send_image_and_dep_data).unwrap();
        context_switch(&thread_ctx).await;
    });
}

pub fn partialt_fix(ctx: &egui::Context, art: &LoadedImage, art_id: ArtEnum, sender: &Sender) {
    // Clone data because we're going to do the heavy listing asyncronously
    //
    let thread_art = art.clone();
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    app_execute(async move {
        let fixed_art = load_image_from_existing_image(
            &thread_art,
            &correct_alpha_for_tshirt,
            "blah_blah_fixed_art", // todo, better name...
            &thread_ctx,
        );
        context_switch(&thread_ctx).await;
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &fixed_art).await;
        let image_to_send = Ok(ImageLoad {
            art_id,
            image: fixed_art,
            dependent_data: Some(dependent_data),
        });
        thread_sender.send(image_to_send).unwrap();
        context_switch(&thread_ctx).await;
    });
}

pub fn cache_in_dependent_data(
    ctx: &egui::Context,
    art: &LoadedImage,
    art_id: ArtEnum,
    sender: &Sender,
) {
    let thread_art = art.clone();
    let thread_ctx = ctx.clone();
    let thread_sender = sender.clone();

    app_execute(async move {
        let dependent_data = ArtworkDependentData::new(&thread_ctx, &thread_art).await;
        let image_to_send = Ok(ImageLoad {
            art_id,
            image: thread_art,
            dependent_data: Some(dependent_data),
        });
        thread_sender.send(image_to_send).unwrap();
        context_switch(&thread_ctx).await;
    });
}
