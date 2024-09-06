use crate::artwork::*;
use crate::error::*;
use crate::image_utils::*;
use crate::loaded_image::*;
use std::future::Future;

// Concurrent pipe and payload definition for asyncronous jobs
pub type AsyncImageLoadResult = Result<AsyncImageLoadPayload, Error>;
pub type AsyncImageSender = std::sync::mpsc::Sender<AsyncImageLoadResult>;
pub type AsyncImageReceiver = std::sync::mpsc::Receiver<AsyncImageLoadResult>;

//
// Asyncronous Image Data Payload
//
// art_id         - The art slot the image is destined for
// art            - The artwork :)
// dependent_data - The artwork's dependent data.  An operation that generates artwork
//                  doesn't have to compute the dependent data.  For example, it may want
//                  to send the core artwork first, so it's visible quickly, and then
//                  compute dependent data later.
//
pub struct AsyncImageLoadPayload {
    pub art_id: ArtEnum,
    pub art: LoadedImage,
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

//
// The main image load/ image function.
//
// Schedules an asyncronous task to do the actual load work so we don't block the main thread.
//
pub fn do_load(
    main_thread_ctx: &egui::Context,
    art_id: ArtEnum,
    main_thread_sender: &AsyncImageSender,
) {
    let ctx = main_thread_ctx.clone();
    let sender = main_thread_sender.clone();

    // Execute the load asyncronously so we don't block the main thread.
    //
    // 1.  Load the image from the user.  Handle any failures
    // 2.  Send the result of that load to the main app so the user sees it quickly
    // 3.  Compute dependent data for the art we just loaded
    // 4.  Send the artwork and the dependent data to the main app
    //
    app_execute(async move {
        // 1.  Load the image from the user.  Handle any failures
        //
        let art_maybe = load_image(&ctx).await;
        if art_maybe.is_err() {
            sender.send(Err(art_maybe.err().unwrap())).unwrap();
            ctx.request_repaint();
            return;
        }
        let art = art_maybe.unwrap();

        // 2.  Send the result of that load to the main app so the user sees it quickly
        //
        context_switch(&ctx).await;
        let send_image = Ok(AsyncImageLoadPayload {
            art_id,
            art: art.clone(),
            dependent_data: None,
        });
        sender.send(send_image).unwrap();

        // 3.  Compute dependent data for the art we just loaded
        //
        context_switch(&ctx).await;
        let dependent_data = ArtworkDependentData::new(&ctx, &art).await;

        // 4.  Send the artwork and the dependent data to the main app
        //
        let send_image_and_dep_data = Ok(AsyncImageLoadPayload {
            art_id,
            art,
            dependent_data: Some(dependent_data),
        });
        sender.send(send_image_and_dep_data).unwrap();
        context_switch(&ctx).await;
    });
}

//
// A utility to fix partial transparency problems.
//
// Schedules an asyncronous task to modify the artwork and update the dependent
// data.
//
pub fn partialt_fix(
    main_thread_ctx: &egui::Context,
    main_thread_art: &LoadedImage,
    art_id: ArtEnum,
    main_thread_sender: &AsyncImageSender,
) {
    //
    // Clone data because we're going to do the heavy listing asyncronously
    //
    let orig_art = main_thread_art.clone();
    let ctx = main_thread_ctx.clone();
    let sender = main_thread_sender.clone();

    app_execute(async move {
        let art = load_image_from_existing_image(
            &orig_art,
            &correct_alpha_for_tshirt,
            "blah_blah_fixed_art", // todo, better name...
            &ctx,
        );
        context_switch(&ctx).await;
        let dependent_data = ArtworkDependentData::new(&ctx, &art).await;
        let image_to_send = Ok(AsyncImageLoadPayload {
            art_id,
            art,
            dependent_data: Some(dependent_data),
        });
        sender.send(image_to_send).unwrap();
        context_switch(&ctx).await;
    });
}

//
// Compute art dependent data asyncronously, then send it to the main thread
//
pub fn cache_in_dependent_data(
    main_thread_ctx: &egui::Context,
    main_thread_art: &LoadedImage,
    art_id: ArtEnum,
    main_thread_sender: &AsyncImageSender,
) {
    let art = main_thread_art.clone();
    let ctx = main_thread_ctx.clone();
    let sender = main_thread_sender.clone();

    app_execute(async move {
        let dependent_data = ArtworkDependentData::new(&ctx, &art).await;
        let image_to_send = Ok(AsyncImageLoadPayload {
            art_id,
            art,
            dependent_data: Some(dependent_data),
        });
        sender.send(image_to_send).unwrap();
        context_switch(&ctx).await;
    });
}
