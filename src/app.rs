use web_time::SystemTime;

extern crate nalgebra as na;
use crate::icons::*;
use crate::image_utils::*;
use crate::loaded_image::*;
use crate::report_templates::*;
use crate::tshirt_storage::*;
use egui_extras::{Size, StripBuilder};
use na::{dvector, matrix, vector, Matrix3, Vector3};

const DEBUG: bool = false;
const TOOL_TOGGLE_RATE: u32 = 500; // in ms
const TOOL_WIDTH: f32 = 20.0;

#[derive(PartialEq, Copy, Clone)]
enum Artwork {
    Artwork0,
    Artwork1,
    Artwork2,
}

pub struct ImageLoad {
    artwork: Artwork,
    image: LoadedImage,
    dependent_data: ArtworkDependentData,
}

pub struct ArtworkDependentData {
    partial_transparency_percent: u32,
    opaque_percent: u32,
    fixed_artwork: LoadedImage,
    flagged_artwork: LoadedImage,
    _heat_map: LoadedImage,
    top_hot_spots: Vec<HotSpot>,
}

impl ArtworkDependentData {
    fn new(ctx: &egui::Context, artwork: &LoadedImage) -> Self {
        let default_fixed_art: LoadedImage = load_image_from_existing_image(
            artwork,
            correct_alpha_for_tshirt,
            "fixed default art",
            ctx,
        );
        let default_flagged_art: LoadedImage = load_image_from_existing_image(
            artwork,
            flag_alpha_for_shirt,
            "flagged default art",
            ctx,
        );
        let heat_map = heat_map_from_image(artwork, "heatmap", ctx);

        Self {
            partial_transparency_percent: compute_bad_tpixels(artwork.pixels()),
            opaque_percent: compute_percent_opaque(artwork.pixels()),
            fixed_artwork: default_fixed_art,
            flagged_artwork: default_flagged_art,
            top_hot_spots: hot_spots_from_heat_map(&heat_map),
            _heat_map: heat_map_from_image(artwork, "heatmap", ctx),
        }
    }
}

pub struct ToolSelection {
    tool_selected_at: SystemTime,
    tool_selected_for: std::option::Option<ReportTypes>,
}

impl ToolSelection {
    fn new() -> Self {
        Self {
            tool_selected_for: None,
            tool_selected_at: SystemTime::now(),
        }
    }
    fn time_since_selection(&self) -> u32 {
        self.tool_selected_at
            .elapsed()
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap()
    }
    fn reset(&mut self) {
        self.tool_selected_for = None;
    }
    fn set(&mut self, tool: ReportTypes, active: bool) {
        if active {
            self.tool_selected_for = Some(tool);
            self.tool_selected_at = SystemTime::now();
        } else {
            self.reset();
        }
    }
    fn get_cycles(&self) -> u32 {
        self.time_since_selection() / TOOL_TOGGLE_RATE
    }
    fn is_active(&self, report_type: ReportTypes) -> bool {
        self.tool_selected_for.is_some() && self.tool_selected_for.unwrap() == report_type
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp {
    artwork_0: LoadedImage,
    artwork_1: LoadedImage,
    artwork_2: LoadedImage,
    art_dependent_data_0: std::option::Option<ArtworkDependentData>,
    art_dependent_data_1: std::option::Option<ArtworkDependentData>,
    art_dependent_data_2: std::option::Option<ArtworkDependentData>,
    icons: IconStorage,
    selected_art: Artwork,
    footer_debug_0: String,
    footer_debug_1: String,
    tshirt_storage: TShirtStorage,
    zoom: f32,
    target: Vector3<f32>,
    last_drag_pos: std::option::Option<Vector3<f32>>,
    drag_display_to_tshirt: std::option::Option<Matrix3<f32>>,
    drag_count: i32,
    tshirt_selected_for: TShirtColors,
    report_templates: ReportTemplates,
    image_loader: Option<std::sync::mpsc::Receiver<Result<ImageLoad, String>>>,
    selected_tool: ToolSelection,
}

fn v3_to_egui(item: Vector3<f32>) -> egui::Pos2 {
    egui::Pos2 {
        x: item.x,
        y: item.y,
    }
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

impl TShirtCheckerApp {
    fn do_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
            if DEBUG {
                ui.horizontal(|ui| {
                    ui.label("footer_debug_0: ");
                    ui.label(&self.footer_debug_0);
                });
                ui.horizontal(|ui| {
                    ui.label("footer_debug_1: ");
                    ui.label(&self.footer_debug_1);
                });
                egui::warn_if_debug_build(ui);
            }
            powered_by_egui_and_eframe(ui);
        });
    }

    fn do_load(&mut self, ctx: &egui::Context) {
        let (sender, receiver) = std::sync::mpsc::channel::<Result<ImageLoad, String>>();
        let art_slot = self.selected_art;
        self.image_loader = Some(receiver);
        let thread_ctx = ctx.clone();
        // Execute in another thread
        app_execute(async move {
            let file = rfd::AsyncFileDialog::new().pick_file().await;
            let data: Vec<u8> = file.unwrap().read().await;

            let image = || -> Result<ImageLoad, String> {
                let image = load_image_from_untrusted_source(&data, "loaded_data", &thread_ctx)?;
                let dependent_data = ArtworkDependentData::new(&thread_ctx, &image);
                Ok(ImageLoad {
                    artwork: art_slot,
                    image,
                    dependent_data,
                })
            };

            sender.send(image()).unwrap();
            thread_ctx.request_repaint();
        });
    }

    fn partialt_fix(&mut self, ctx: &egui::Context) {
        let art = self.get_selected_art();
        let fixed_art = load_image_from_existing_image(
            art,
            |p| {
                let new_alpha: u8 = if p.a() < 25 { 0 } else { 255 };
                egui::Color32::from_rgba_premultiplied(p.r(), p.g(), p.b(), new_alpha)
            },
            "fixed_art", // todo, better name...
            ctx,
        );
        let dependent_data = ArtworkDependentData::new(ctx, &fixed_art);
        self.set_artwork(self.selected_art, fixed_art, dependent_data);
    }

    //
    // Transforms from "t shirt space", where (0,0) is the top
    // left corner of the t shirt image and (1,1) is the bottom
    // right corner of the t-shirt image, to the display.
    //
    fn tshirt_to_display(&self, panel_size: egui::Vec2) -> Matrix3<f32> {
        let panel_aspect = panel_size[0] / panel_size[1];

        let tshirt_size = self.tshirt_storage.size();
        let tshirt_aspect = tshirt_size.x / tshirt_size.y;

        let move_from_center: Matrix3<f32> = matrix![ 1.0,  0.0,  -self.target.x;
                     0.0,  1.0,  -self.target.y;
                     0.0,  0.0,  1.0 ];
        let move_to_center: Matrix3<f32> = matrix![ 1.0,  0.0,  0.5;
                     0.0,  1.0,  0.5;
                     0.0,  0.0,  1.0 ];
        let scale: Matrix3<f32> = matrix![ self.zoom,  0.0,        0.0;
                     0.0,        self.zoom,  0.0;
                     0.0,        0.0,        1.0 ];

        let scale_centered = move_to_center * scale * move_from_center;

        if panel_aspect > tshirt_aspect {
            // panel is wider than the t-shirt
            let x_width = panel_size[0] * tshirt_aspect / panel_aspect;
            let x_margin = (panel_size[0] - x_width) / 2.0;
            return matrix![  x_width,    0.0,             x_margin;
                             0.0,        panel_size[1],   0.0;
                             0.0,        0.0,             1.0  ]
                * scale_centered;
        }
        // panel is higher than the t-shirt
        let y_width = panel_size[1] / tshirt_aspect * panel_aspect;
        let y_margin = (panel_size[1] - y_width) / 2.0;
        matrix![  panel_size[0],    0.0,             0.0;
                  0.0,              y_width,         y_margin;
                  0.0,              0.0,             1.0  ]
            * scale_centered
    }

    fn art_enum_to_image(&self, artwork: Artwork) -> &LoadedImage {
        match artwork {
            Artwork::Artwork0 => &self.artwork_0,
            Artwork::Artwork1 => &self.artwork_1,
            Artwork::Artwork2 => &self.artwork_2,
        }
    }

    fn art_enum_to_dependent_data(&self, artwork: Artwork) -> &ArtworkDependentData {
        // For now I guess I guarantee, through logic that's hard to reason about
        // that the unwrap always succeeds.  Definately a comments are a code
        // smell moment.
        match artwork {
            Artwork::Artwork0 => self.art_dependent_data_0.as_ref().unwrap(),
            Artwork::Artwork1 => self.art_dependent_data_1.as_ref().unwrap(),
            Artwork::Artwork2 => self.art_dependent_data_2.as_ref().unwrap(),
        }
    }

    fn cache_in_art_dependent_data(&mut self, ctx: &egui::Context, artwork: Artwork) {
        let image: &LoadedImage = self.art_enum_to_image(artwork);

        match artwork {
            Artwork::Artwork0 => {
                if self.art_dependent_data_0.is_none() {
                    self.art_dependent_data_0 = Some(ArtworkDependentData::new(ctx, image));
                }
            }
            Artwork::Artwork1 => {
                if self.art_dependent_data_1.is_none() {
                    self.art_dependent_data_1 = Some(ArtworkDependentData::new(ctx, image));
                }
            }
            Artwork::Artwork2 => {
                if self.art_dependent_data_2.is_none() {
                    self.art_dependent_data_2 = Some(ArtworkDependentData::new(ctx, image));
                }
            }
        }
    }

    fn get_selected_art(&self) -> &LoadedImage {
        self.art_enum_to_image(self.selected_art)
    }

    fn art_to_art_space(art: &LoadedImage) -> Matrix3<f32> {
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
    fn art_space_to_tshirt(&self) -> Matrix3<f32> {
        let tshirt_size = self.tshirt_storage.size();
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

    fn handle_central_movement_drag(
        &mut self,
        response: &egui::Response,
        panel_size: egui::Vec2,
    ) -> bool {
        let mut movement_attempted = false;

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let current_drag_pos = vector!(pointer_pos[0], pointer_pos[1], 1.0);

            if let Some(last_drag_pos) = self.last_drag_pos {
                let display_to_artspace = self.drag_display_to_tshirt.unwrap();
                let last = display_to_artspace * last_drag_pos;
                let curr = display_to_artspace * current_drag_pos;
                self.target = self.target + last - curr;
                movement_attempted = true;
            } else {
                let tshirt_to_display = self.tshirt_to_display(panel_size);
                self.drag_display_to_tshirt = Some(tshirt_to_display.try_inverse().unwrap());
                self.drag_count += 1;
            }
            self.last_drag_pos = Some(current_drag_pos);
        } else {
            self.last_drag_pos = None;
            self.drag_display_to_tshirt = None;
        }
        movement_attempted
    }

    fn handle_central_movement_zoom(&mut self, ui: &egui::Ui, response: &egui::Response) -> bool {
        let mut movement_attempted = false;

        if response.hovered() {
            let zoom_delta_0 = 1.0 + ui.ctx().input(|i| i.smooth_scroll_delta)[1] / 200.0;
            let zoom_delta_1 = ui.ctx().input(|i| i.zoom_delta());
            let zoom_delta = if zoom_delta_0 != 1.0 {
                zoom_delta_0
            } else {
                zoom_delta_1
            };
            if zoom_delta != 1.0 {
                movement_attempted = true;
            }

            self.zoom *= zoom_delta;
            if self.zoom < 1.0 {
                self.zoom = 1.0;
            }
        }

        movement_attempted
    }

    fn handle_central_movement(
        &mut self,
        ui: &egui::Ui,
        response: egui::Response,
        panel_size: egui::Vec2,
    ) -> bool {
        let dragged = self.handle_central_movement_drag(&response, panel_size);
        let zoomed = self.handle_central_movement_zoom(ui, &response);

        dragged || zoomed
    }

    fn paint_tshirt(&self, painter: &egui::Painter, panel_size: egui::Vec2) {
        let tshirt_to_display = self.tshirt_to_display(panel_size);

        let uv0 = egui::Pos2 { x: 0.0, y: 0.0 };
        let uv1 = egui::Pos2 { x: 1.0, y: 1.0 };

        let s0 = v3_to_egui(tshirt_to_display * dvector![0.0, 0.0, 1.0]);
        let s1 = v3_to_egui(tshirt_to_display * dvector![1.0, 1.0, 1.0]);

        let tshirt_art = self
            .tshirt_storage
            .tshirt_enum_to_image(self.tshirt_selected_for);

        painter.image(
            tshirt_art.id(),
            egui::Rect::from_min_max(s0, s1),
            egui::Rect::from_min_max(uv0, uv1),
            egui::Color32::WHITE,
        );
    }

    fn paint_artwork(&self, painter: &egui::Painter, panel_size: egui::Vec2) {
        let tshirt_to_display = self.tshirt_to_display(panel_size);
        let art = self.get_selected_art();
        let art_space_to_display = tshirt_to_display * self.art_space_to_tshirt();
        let art_to_display = art_space_to_display * Self::art_to_art_space(art);

        let a0 = v3_to_egui(art_to_display * dvector![0.0, 0.0, 1.0]);
        let a1 = v3_to_egui(art_to_display * dvector![1.0, 1.0, 1.0]);
        let uv0 = egui::Pos2 { x: 0.0, y: 0.0 };
        let uv1 = egui::Pos2 { x: 1.0, y: 1.0 };

        let cycle = self.selected_tool.get_cycles() % 2;
        let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
        let texture_to_display = if self
            .selected_tool
            .is_active(ReportTypes::PartialTransparency)
        {
            match cycle {
                0 => dependent_data.flagged_artwork.id(),
                _ => dependent_data.fixed_artwork.id(),
            }
        } else if self.selected_tool.is_active(ReportTypes::Dpi) {
            dependent_data.fixed_artwork.id()
        } else {
            self.get_selected_art().id()
        };

        painter.image(
            texture_to_display,
            egui::Rect::from_min_max(a0, a1),
            egui::Rect::from_min_max(uv0, uv1),
            egui::Color32::WHITE,
        );
    }

    fn do_dpi_tool(&mut self, movement_happened: bool) {
        let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
        let cycle = self.selected_tool.get_cycles() / 10;
        let slot = cycle % (dependent_data.top_hot_spots.len() as u32);
        let hot_spot = &dependent_data.top_hot_spots[slot as usize];
        let art_location = vector![hot_spot.location.x, hot_spot.location.y, 1.0];
        let art = self.get_selected_art();
        let art_to_tshirt = self.art_space_to_tshirt() * Self::art_to_art_space(art);
        let display_location = art_to_tshirt * art_location;

        // need to make modifications to self after dependent_data borrow is done.
        if !movement_happened {
            self.zoom = 10.0;
            self.target = display_location;
        } else {
            // deselect tool if the user is trying to move or zoom.
            self.selected_tool.reset();
        }
    }

    fn paint_area_used_tool(&self, painter: &egui::Painter, panel_size: egui::Vec2) {
        let tshirt_to_display = self.tshirt_to_display(panel_size);
        let art_space_to_display = tshirt_to_display * self.art_space_to_tshirt();
        let art_space_border = vec![
            v3_to_egui(art_space_to_display * dvector![0.0, 0.0, 1.0]),
            v3_to_egui(art_space_to_display * dvector![11.0, 0.0, 1.0]),
            v3_to_egui(art_space_to_display * dvector![11.0, 14.0, 1.0]),
            v3_to_egui(art_space_to_display * dvector![0.0, 14.0, 1.0]),
            v3_to_egui(art_space_to_display * dvector![0.0, 0.0, 1.0]),
        ];

        let dash_dim = (art_space_to_display * dvector![0.2, 0.05, 1.0])
            - (art_space_to_display * dvector![0.0, 0.0, 1.0]);
        let dash_length = dash_dim.x;
        let dash_width = dash_dim.y;
        let gap_length = dash_length;

        // animate with 3 cycles
        let cycle = self.selected_tool.get_cycles() % 3;
        let offset: f32 = (cycle as f32) / 3.0 * (dash_length + gap_length);
        let stroke_1 = egui::Stroke::new(dash_width, egui::Color32::from_rgb(200, 200, 200));

        painter.add(egui::Shape::dashed_line_with_offset(
            &art_space_border,
            stroke_1,
            &[dash_length],
            &[gap_length],
            offset,
        ));
    }

    fn do_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_size = ui.available_size_before_wrap();
            let (response, painter) =
                ui.allocate_painter(panel_size, egui::Sense::click_and_drag());

            let movement_happened = self.handle_central_movement(ui, response, panel_size);
            self.paint_tshirt(&painter, panel_size);
            self.paint_artwork(&painter, panel_size);

            if self.selected_tool.is_active(ReportTypes::Dpi) {
                self.do_dpi_tool(movement_happened);
            }
            if self.selected_tool.is_active(ReportTypes::AreaUsed) {
                self.paint_area_used_tool(&painter, panel_size);
            }
        });
    }

    fn handle_tshirt_button(&mut self, ui: &mut egui::Ui, color: TShirtColors) {
        let image: &LoadedImage = self.tshirt_storage.tshirt_enum_to_image(color);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(80.0);
        let is_selected = self.tshirt_selected_for == color;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            self.tshirt_selected_for = color;
        }
    }

    fn handle_art_button(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, artwork: Artwork) {
        let image: &LoadedImage = self.art_enum_to_image(artwork);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(80.0);
        let is_selected = self.selected_art == artwork;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            self.cache_in_art_dependent_data(ctx, artwork);
            self.selected_art = artwork;
            self.selected_tool.reset();
        }
    }

    fn compute_dpi(art: &LoadedImage) -> u32 {
        let top_corner = Self::art_to_art_space(art) * dvector![0.0, 0.0, 1.0];
        let bot_corner = Self::art_to_art_space(art) * dvector![1.0, 1.0, 1.0];
        let dim_in_inches = bot_corner - top_corner;
        (art.size().x / dim_in_inches.x) as u32
    }

    fn compute_badtransparency_pixels(&self) -> u32 {
        let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
        dependent_data.partial_transparency_percent
    }

    fn compute_area_used(art: &LoadedImage) -> u32 {
        let top_corner = Self::art_to_art_space(art) * dvector![0.0, 0.0, 1.0];
        let bot_corner = Self::art_to_art_space(art) * dvector![1.0, 1.0, 1.0];
        let dim_in_inches = bot_corner - top_corner;
        let area_used = 100.0 * dim_in_inches[0] * dim_in_inches[1] / (11.0 * 14.0);
        area_used as u32
    }

    fn compute_bib_score(&self) -> u32 {
        let art = self.get_selected_art();
        let area_used = Self::compute_area_used(art);
        let dependent_data = self.art_enum_to_dependent_data(self.selected_art);
        area_used * dependent_data.opaque_percent / 100
    }

    fn report_metric(&mut self, ui: &mut egui::Ui, report_type: ReportTypes, metric: u32) {
        ui.horizontal(|ui| {
            StripBuilder::new(ui)
                .size(Size::exact(25.0))
                .size(Size::exact(140.0))
                .size(Size::exact(40.0))
                .size(Size::exact(15.0))
                .size(Size::exact(TOOL_WIDTH))
                .horizontal(|mut strip| {
                    let report = self.report_templates.report_type_to_template(report_type);
                    let status = (report.metric_to_status)(metric);
                    let status_icon = self.icons.status_icon(status);
                    let tool_tip = report.tool_tip.clone();
                    let report_tip = report.report_tip.clone();

                    strip.cell(|ui| {
                        ui.add(status_icon).on_hover_text(&report_tip);
                    });
                    strip.cell(|ui| {
                        ui.label(mtexts(&report.label.to_string()))
                            .on_hover_text(&report_tip);
                    });
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(mtexts(&format!("{}", metric)))
                                .on_hover_text(&report_tip);
                        });
                    });
                    let cell_string = (if report.display_percent { "%" } else { "" }).to_string();
                    strip.cell(|ui| {
                        ui.label(mtexts(&cell_string));
                    });
                    strip.cell(|ui| {
                        if status != ReportStatus::Pass {
                            let is_selected = self.selected_tool.is_active(report_type);
                            if ui
                                .add(
                                    self.icons
                                        .button(Icon::Tool, TOOL_WIDTH)
                                        .selected(is_selected),
                                )
                                .on_hover_text(tool_tip)
                                .clicked()
                            {
                                self.selected_tool.set(report_type, !is_selected);
                            }
                        }
                    });
                });
        });
    }

    fn panel_separator(ui: &mut egui::Ui) {
        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);
    }

    fn display_title(ui: &mut egui::Ui) {
        Self::panel_separator(ui);
        ui.vertical_centered(|ui| {
            ui.heading(egui::widget_text::RichText::from("T-Shirt Art Checker").size(30.0))
        });
        Self::panel_separator(ui);
    }

    fn report_metrics(&mut self, ui: &mut egui::Ui) {
        self.report_metric(
            ui,
            ReportTypes::Dpi,
            Self::compute_dpi(self.get_selected_art()),
        );
        self.report_metric(
            ui,
            ReportTypes::AreaUsed,
            Self::compute_area_used(self.get_selected_art()),
        );
        self.report_metric(ui, ReportTypes::Bib, self.compute_bib_score());
        self.report_metric(
            ui,
            ReportTypes::PartialTransparency,
            self.compute_badtransparency_pixels(),
        );
        Self::panel_separator(ui);
    }

    fn tshirt_selection_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.handle_tshirt_button(ui, TShirtColors::Red);
            self.handle_tshirt_button(ui, TShirtColors::Green);
            self.handle_tshirt_button(ui, TShirtColors::Blue);
        });
        ui.horizontal(|ui| {
            self.handle_tshirt_button(ui, TShirtColors::DRed);
            self.handle_tshirt_button(ui, TShirtColors::DGreen);
            self.handle_tshirt_button(ui, TShirtColors::DBlue);
        });
        Self::panel_separator(ui);
    }

    fn artwork_selection_panel(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            self.handle_art_button(ctx, ui, Artwork::Artwork0);
            self.handle_art_button(ctx, ui, Artwork::Artwork1);
            self.handle_art_button(ctx, ui, Artwork::Artwork2);
        });
        Self::panel_separator(ui);
    }

    fn import_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui
            .add(self.icons.button(Icon::Import, 80.0))
            .on_hover_text("Import an image to the selected artwork slot.")
            .clicked()
        {
            self.do_load(ctx);
        }
    }

    fn partial_transparency_fix_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ui
            .add(self.icons.button(Icon::FixPT, 80.0))
            .on_hover_text(
                "Fix partial transparency problems by mapping all alpha values to 0 or 1.",
            )
            .clicked()
        {
            self.partialt_fix(ctx);
        }
    }

    fn set_artwork(
        &mut self,
        slot: Artwork,
        image: LoadedImage,
        dependent_data: ArtworkDependentData,
    ) {
        match slot {
            Artwork::Artwork0 => {
                self.artwork_0 = image;
                self.art_dependent_data_0 = Some(dependent_data);
            }
            Artwork::Artwork1 => {
                self.artwork_1 = image;
                self.art_dependent_data_1 = Some(dependent_data);
            }
            Artwork::Artwork2 => {
                self.artwork_2 = image;
                self.art_dependent_data_2 = Some(dependent_data);
            }
        }
    }

    fn do_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("stuff")
            .resizable(true)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    Self::display_title(ui);
                    self.report_metrics(ui);
                    self.tshirt_selection_panel(ui);
                    self.artwork_selection_panel(ui, ctx);

                    ui.horizontal(|ui| {
                        self.import_button(ui, ctx);
                        self.partial_transparency_fix_button(ui, ctx);
                    });
                })
            });
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let artwork_0: LoadedImage = load_image_from_trusted_source(
            include_bytes!("test_artwork.png"),
            "artwork_0",
            &cc.egui_ctx,
        );
        let artwork_1: LoadedImage = load_image_from_trusted_source(
            include_bytes!("sf2024-attendee-v1.png"),
            "artwork_1",
            &cc.egui_ctx,
        );
        let artwork_2: LoadedImage = load_image_from_trusted_source(
            include_bytes!("sf2024-attendee-v2.png"),
            "artwork_2",
            &cc.egui_ctx,
        );

        Self {
            art_dependent_data_0: Some(ArtworkDependentData::new(&cc.egui_ctx, &artwork_0)),
            art_dependent_data_1: None,
            art_dependent_data_2: None,
            selected_art: Artwork::Artwork0,
            footer_debug_0: String::new(),
            footer_debug_1: String::new(),
            tshirt_storage: TShirtStorage::new(&cc.egui_ctx),
            icons: IconStorage::new(&cc.egui_ctx),
            zoom: 1.0,
            target: vector![0.50, 0.50, 1.0],
            last_drag_pos: None,
            drag_display_to_tshirt: None,
            drag_count: 0,
            tshirt_selected_for: TShirtColors::Red,
            artwork_0,
            artwork_1,
            artwork_2,
            report_templates: ReportTemplates::new(),
            image_loader: None,
            selected_tool: ToolSelection::new(),
        }
    }
}

fn mtexts(text: &String) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0)
}

impl eframe::App for TShirtCheckerApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.footer_debug_0 = format!("time {}", self.selected_tool.time_since_selection());
        if self.image_loader.is_some() {
            let rcv = self.image_loader.as_ref().unwrap();
            let data_attempt = rcv.try_recv();
            if data_attempt.is_ok() {
                let loaded_result = data_attempt.unwrap();
                match loaded_result {
                    Err(e) => {
                        self.footer_debug_1 = format!("Error: {}", e);
                    }
                    Ok(f) => {
                        self.set_artwork(f.artwork, f.image, f.dependent_data);
                    }
                }
                self.image_loader = None;
            }
        }
        self.do_bottom_panel(ctx);
        self.do_right_panel(ctx);
        self.do_central_panel(ctx);
        if self
            .selected_tool
            .is_active(ReportTypes::PartialTransparency)
            || self.selected_tool.is_active(ReportTypes::AreaUsed)
            || self.selected_tool.is_active(ReportTypes::Dpi)
        {
            let time_in_ms = self.selected_tool.time_since_selection();
            let next_epoch = (time_in_ms / TOOL_TOGGLE_RATE + 1) * TOOL_TOGGLE_RATE + 1;
            let time_to_wait = next_epoch - time_in_ms;

            ctx.request_repaint_after(std::time::Duration::from_millis(time_to_wait.into()))
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
