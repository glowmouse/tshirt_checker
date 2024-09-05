extern crate nalgebra as na;
use crate::artwork::*;
use crate::error::*;
use crate::icons::*;
use crate::loaded_image::*;
use crate::math::*;
use crate::movement_state::MovementState;
use crate::notice_panel::*;
use crate::report_templates::*;
use crate::tool_select::*;
use crate::tshirt_storage::*;
use egui_extras::{Size, StripBuilder};
use na::{dvector, vector, Matrix3};

const TOOL_WIDTH: f32 = 20.0;
const STATUS_ICON_WIDTH: f32 = 25.0;
const REPORT_TEXT_WIDTH: f32 = 150.0;
const REPORT_METRIC_WIDTH: f32 = 40.0;
const REPORT_PERCENT_WIDTH: f32 = 25.0;
const BUTTON_WIDTH: f32 = 80.0;

// State for the TShirt Artwork Checker app
//
pub struct TShirtCheckerApp {
    // Storage for artwork the user may want to put on the t-shiurt
    art_storage: ArtStorage,
    // Which of the 4 pieces of artwork from art_storage is currently selected.
    selected_art: Artwork,
    // Storage for icons used by the app
    icons: IconStorage,
    // Persistant state used to move or zoom the t-shirt and artwork
    move_state: MovementState,
    // Storage for the different color t-shirt images
    tshirt_image_storage: TShirtStorage,
    // Color of the tshirt that's currently selected
    selected_tshirt: TShirtColors,
    // Template storage for the different tshirt arts report types
    report_templates: ReportTemplates,
    // Sender and receiver for image data that's computed asyncronously to improve load times
    async_data_to_app_sender: crate::async_tasks::Sender,
    async_data_to_app_receiver: crate::async_tasks::Receiver,
    // What artwork tool is selected
    selected_tool: ToolSelection,
    // Bottom notification panel.  Used for changes like image load failures
    notification_panel: NoticePanel,
}

//
// T-Shirt checker does its updating with TShirtCheckerApp in a read only state
// Any changes that need to be made to the app are added to ChangesToBeMade and then made
// when the update is done.
//
// It's a bit easier to reason about in the sense that the update is the most complicated
// thing that's happening here - one part of the update won't interfer in a weird
// way with another part of the update by modifying the TShirtCheckerApp state.
//
pub type ChangeToBeMade = Box<dyn Fn(&mut TShirtCheckerApp)>;

#[derive(Default)]
pub struct ChangesToBeMade {
    changes: Vec<ChangeToBeMade>,
}

impl std::ops::AddAssign<ChangeToBeMade> for &mut ChangesToBeMade {
    fn add_assign(&mut self, rhs: ChangeToBeMade) {
        self.changes.push(rhs);
    }
}

impl std::ops::AddAssign<ChangeToBeMade> for ChangesToBeMade {
    fn add_assign(&mut self, rhs: ChangeToBeMade) {
        self.changes.push(rhs);
    }
}

// TShirt Checker App entry point for updates
impl eframe::App for TShirtCheckerApp {
    //
    // Eframe's hook for updating the application.
    //
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let changes = self.paint_all_panels(ctx);
        self.journal_changes_to_app_state(changes);
        self.schedule_repaint_request_if_needed(ctx);
    }
}

impl TShirtCheckerApp {
    //
    // Single TShirtCheckerApp creation point.
    //
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        //
        // load the default artwork.  This will involved 3 in memory PNG decompressions,
        // which isn't super best practice runtime wise. TODO, measure actual time.
        //
        let art_storage = ArtStorage::new(&cc.egui_ctx);

        //
        // Choose the artwork in "slot 0" to be the initial selected art.
        //
        let selected_art = Artwork::Artwork0;

        //
        // Create a pipe that will be used by computationally heavy tasks so we don't
        // block the main thread during updates
        //
        let (async_data_to_app_sender, async_data_to_app_receiver) =
            std::sync::mpsc::channel::<crate::async_tasks::Payload>();

        //
        // Schedule an asychronous task to create the artwork needed to display
        // any reports for our initial piece of selected artwork.
        //
        crate::async_tasks::cache_in_dependent_data(
            &cc.egui_ctx,
            art_storage.get_art(selected_art),
            selected_art,
            &async_data_to_app_sender,
        );

        Self {
            art_storage,
            selected_art,
            tshirt_image_storage: TShirtStorage::new(&cc.egui_ctx),
            move_state: MovementState::new(),
            icons: IconStorage::new(&cc.egui_ctx),
            selected_tshirt: TShirtColors::Red,
            report_templates: ReportTemplates::new(),
            selected_tool: ToolSelection::new(),
            async_data_to_app_sender,
            async_data_to_app_receiver,
            notification_panel: NoticePanel::new(),
        }
    }

    // Paint everything in the GUI
    //
    fn paint_all_panels(&self, ctx: &egui::Context) -> ChangesToBeMade {
        let mut changes: ChangesToBeMade = Default::default();
        self.paint_bottom_panel(ctx);
        self.paint_right_panel(&mut changes, ctx);
        self.paint_central_panel(&mut changes, ctx);
        changes
    }

    // Display the bottom panel in the app - notifications & powered by egui and eframe
    //
    fn paint_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
            self.notification_panel.display(ui);
            Self::powered_by_egui_and_eframe(ui);
        });
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

    ////////////////////////////////////////////////////////////////////////////////////
    //
    // Paint the right panel (tool status, artwork and t-shirt selection
    //
    // Concepts in the code that displays the right panel...
    //
    // size - These are floats from 0.2 to 1.0 that tell the paint code how much it
    //        should try to scale down panel during drawing.  1.0 is full size and
    //        should be displayable on a 780 x 900 screen.  If the screen is smaller
    //        (i.e.,  a cell phone display or something) the target size gets reduced
    //        so the entire app fits on the screen.
    //
    // changes - This is a list of all the changes that need to be made to
    //        the application state after everything is painted.  All paints in the app
    //        are read only.
    //
    ////////////////////////////////////////////////////////////////////////////////////

    // Paint the panel on the right hand side.
    //
    fn paint_right_panel(&self, changes: &mut ChangesToBeMade, ctx: &egui::Context) {
        let screen = ctx.screen_rect();
        let size = screen.max - screen.min;
        let scale_x = (size.x * 0.33) / 260.0;
        let scale_y = (size.y - 150.0) / 700.0;
        let scale = scale_x.min(scale_y).clamp(0.20, 1.0);

        let targetx = 50.0 + 260.0 * scale;

        egui::SidePanel::right("stuff")
            .resizable(true)
            .min_width(targetx)
            .max_width(targetx)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    self.paint_title(ui, scale);
                    self.paint_reports(changes, ui, scale);
                    self.paint_tshirt_selection_panel(changes, ui, scale);
                    self.paint_artwork_selection_panel(changes, ui, ctx, scale);

                    ui.horizontal(|ui| {
                        self.paint_import_button(ui, ctx, scale);
                        self.paint_partial_transparency_fix_button(ui, ctx, scale);
                    });
                })
            });
    }

    // The t-squared title and logo
    //
    fn paint_title(&self, ui: &mut egui::Ui, scale: f32) {
        Self::paint_panel_separator(ui, scale);
        let width = 35.0 * scale;
        let logo = self.icons.image(Icon::Logo, 85.0 * scale);
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add(logo);
                ui.add_space(10.0 * scale);
                ui.vertical(|ui| {
                    ui.heading(egui::widget_text::RichText::from("T-Shirt Art").size(width));
                    ui.heading(egui::widget_text::RichText::from("Checker").size(width))
                });
            });
        });
        Self::paint_panel_separator(ui, scale);
    }

    // Display the 5 reports for the selected artwork
    //
    fn paint_reports(&self, changes: &mut ChangesToBeMade, ui: &mut egui::Ui, scale: f32) {
        self.paint_report(changes, ui, scale, ReportTypes::Dpi);
        self.paint_report(changes, ui, scale, ReportTypes::AreaUsed);
        self.paint_report(changes, ui, scale, ReportTypes::Bib);
        self.paint_report(changes, ui, scale, ReportTypes::ThinLines);
        self.paint_report(changes, ui, scale, ReportTypes::PartialTransparency);

        Self::paint_panel_separator(ui, scale);
    }

    // Paint one report line
    //
    fn paint_report(
        &self,
        changes: &mut ChangesToBeMade,
        ui: &mut egui::Ui,
        scale: f32,
        report_type: ReportTypes,
    ) {
        let report_template = self.report_templates.report_type_to_template(report_type);
        let art = self.get_selected_art();
        let dependent_data = self.get_selected_dependent_data();

        // Column 1 - Name of the report
        let report_name = mtexts(&report_template.label, scale);

        // Column 2 - The status of the report (i.e., pass/ warn, fail)
        let status_icon = self
            .icons
            .status_icon(report_template.status(art, dependent_data))
            .max_width(STATUS_ICON_WIDTH * scale);

        // Column 3 - The text for the score of the report's metric
        let metric_text = report_template.metric_text(art, dependent_data);
        let metric = mtexts(&metric_text, scale);

        // Column 4 - The postfix after the matrix (either a % or nothing)
        let metrix_postfix = mtexts(&report_template.postfix_string(), scale);

        ui.horizontal(|ui| {
            StripBuilder::new(ui)
                .size(Size::exact(STATUS_ICON_WIDTH * scale))
                .size(Size::exact(REPORT_TEXT_WIDTH * scale))
                .size(Size::exact(REPORT_METRIC_WIDTH * scale))
                .size(Size::exact(REPORT_PERCENT_WIDTH * scale))
                .size(Size::exact(TOOL_WIDTH * scale))
                .horizontal(|mut strip| {
                    let report_tip = &(report_template.report_tip);

                    // Column 1 - Name of the report
                    strip.cell(|ui| {
                        ui.add(status_icon).on_hover_text(report_tip);
                    });

                    // Column 2 - The status of the report (i.e., pass/ warn, fail)
                    strip.cell(|ui| {
                        ui.label(report_name).on_hover_text(report_tip);
                    });

                    // Column 3 - The text for the score of the report's metric
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            ui.label(metric).on_hover_text(report_tip);
                        });
                    });

                    // Column 4 - The postfix after the matrix (either a % or nothing)
                    strip.cell(|ui| {
                        ui.label(metrix_postfix);
                    });

                    // Column 5 - The tool select button (painted separately)
                    strip.cell(|ui| {
                        let status = report_template.status(art, dependent_data);
                        self.paint_tool_button(changes, ui, scale, report_type, status);
                    });
                });
        });
    }

    // Paint the button that turns the report's helper tool on or off
    //
    fn paint_tool_button(
        &self,
        mut changes: &mut ChangesToBeMade,
        ui: &mut egui::Ui,
        scale: f32,
        report_type: ReportTypes,
        status: ReportStatus,
    ) {
        // Skip if there's no problem or if we're still computing report data
        let skip_button_paint = status == ReportStatus::Pass || status == ReportStatus::Unknown;
        if skip_button_paint {
            return;
        }

        // Grab tool tip and "is the report already selected (running) status
        let report_template = self.report_templates.report_type_to_template(report_type);
        let tool_tip = &(report_template.tool_tip);
        let is_selected = self.selected_tool.is_active(report_type);

        // Paint the button
        if ui
            .add(
                self.icons
                    .button(Icon::Tool, TOOL_WIDTH * scale)
                    .selected(is_selected),
            )
            .on_hover_text(tool_tip)
            .clicked()
        {
            // If the button's clicked, schedule a tool select event.
            changes += Box::new(move |app: &mut Self| {
                app.selected_tool.set(report_type, !is_selected);
            });
        }
    }

    // Paint all the t-hsirts in two rows
    //
    fn paint_tshirt_selection_panel(
        &self,
        changes: &mut ChangesToBeMade,
        ui: &mut egui::Ui,
        scale: f32,
    ) {
        ui.horizontal(|ui| {
            self.paint_tshirt_select_button(changes, ui, scale, TShirtColors::Red);
            self.paint_tshirt_select_button(changes, ui, scale, TShirtColors::Green);
            self.paint_tshirt_select_button(changes, ui, scale, TShirtColors::Blue);
        });
        ui.horizontal(|ui| {
            self.paint_tshirt_select_button(changes, ui, scale, TShirtColors::DRed);
            self.paint_tshirt_select_button(changes, ui, scale, TShirtColors::DGreen);
            self.paint_tshirt_select_button(changes, ui, scale, TShirtColors::DBlue);
        });
        Self::paint_panel_separator(ui, scale);
    }

    // Paint one t-shirt
    //
    fn paint_tshirt_select_button(
        &self,
        mut changes: &mut ChangesToBeMade,
        ui: &mut egui::Ui,
        scale: f32,
        color: TShirtColors,
    ) {
        let width = BUTTON_WIDTH * scale;
        let image: &LoadedImage = self.tshirt_image_storage.tshirt_enum_to_image(color);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(width);
        let is_selected = self.selected_tshirt == color;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            // If the t-shirt button is clicked, schedule the t-shirt change
            changes += Box::new(move |app: &mut Self| {
                app.selected_tshirt = color;
            });
        }
    }

    // Paint the three potential pieces of art-work
    //
    fn paint_artwork_selection_panel(
        &self,
        changes: &mut ChangesToBeMade,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
    ) {
        ui.horizontal(|ui| {
            self.paint_art_select_button(changes, ui, ctx, scale, Artwork::Artwork0);
            self.paint_art_select_button(changes, ui, ctx, scale, Artwork::Artwork1);
            self.paint_art_select_button(changes, ui, ctx, scale, Artwork::Artwork2);
        });
        Self::paint_panel_separator(ui, scale);
    }

    //
    // Paint the button for one piece of artwork
    //
    fn paint_art_select_button(
        &self,
        mut changes: &mut ChangesToBeMade,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
        artwork: Artwork,
    ) {
        let width = BUTTON_WIDTH * scale;
        let image: &LoadedImage = self.art_storage.get_art(artwork);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(width);
        let is_selected = self.selected_art == artwork;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            if self.art_storage.get_dependent_data(artwork).is_none() {
                // schedule compute of cached data needed for reports.
                // Some reports may not be available until the computation finishes
                //
                // TODO - if somebody spam clicks this we'll spam schedule an expensive
                // and needless recomputation task.
                //
                crate::async_tasks::cache_in_dependent_data(
                    ctx,
                    self.art_storage.get_art(artwork),
                    artwork,
                    &self.async_data_to_app_sender,
                );
            }
            // Schedule the artwork change after the paint is done
            changes += Box::new(move |app: &mut Self| {
                app.selected_art = artwork;
                app.selected_tool.reset();
            });
        }
    }

    // The import button (so people can load their own artwork)
    //
    fn paint_import_button(&self, ui: &mut egui::Ui, ctx: &egui::Context, scale: f32) {
        let width = BUTTON_WIDTH * scale;
        if ui
            .add(self.icons.button(Icon::Import, width))
            .on_hover_text("Import an image to the selected artwork slot.")
            .clicked()
        {
            // Start an asyncronous load task
            crate::async_tasks::do_load(ctx, self.selected_art, &self.async_data_to_app_sender);
        }
    }

    // A simple tool to fix partial transparency problems
    //
    fn paint_partial_transparency_fix_button(
        &self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
    ) {
        let width = BUTTON_WIDTH * scale;
        if ui
            .add(self.icons.button(Icon::FixPT, width))
            .on_hover_text(
                "Fix partial transparency problems by mapping all alpha values to 0 or 1.",
            )
            .clicked()
        {
            // Start the partial transparency fix asyncronously.
            crate::async_tasks::partialt_fix(
                ctx,
                self.art_storage.get_art(self.selected_art),
                self.selected_art,
                &self.async_data_to_app_sender,
            );
        }
    }

    // A separator for the panels on the right hand side.
    //
    fn paint_panel_separator(ui: &mut egui::Ui, scale: f32) {
        ui.add_space(5.0 * scale);
        ui.separator();
        ui.add_space(5.0 * scale);
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    // Paint the central panel (active t-shirt, art, and any tool output
    //
    ////////////////////////////////////////////////////////////////////////////////////

    fn paint_central_panel(&self, changes: &mut ChangesToBeMade, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let display_size = ui.available_size_before_wrap();
            let (response, painter) =
                ui.allocate_painter(display_size, egui::Sense::click_and_drag());

            let movement_happened =
                self.handle_central_movement(changes, ui, response, display_size);
            self.paint_tshirt(&painter, display_size);
            self.paint_artwork(&painter, display_size);

            if self.selected_tool.is_active(ReportTypes::Dpi) {
                self.paint_dpi_tool(changes, movement_happened);
            }
            if self.selected_tool.is_active(ReportTypes::AreaUsed) {
                self.paint_area_used_tool(&painter, display_size);
            }
        });
    }

    fn handle_central_movement(
        &self,
        changes: &mut ChangesToBeMade,
        ui: &egui::Ui,
        response: egui::Response,
        display_size: egui::Vec2,
    ) -> bool {
        let dragged = self.handle_central_movement_drag(changes, &response, display_size);
        let zoomed = self.handle_central_movement_zoom(changes, ui, &response);

        dragged || zoomed
    }

    fn handle_central_movement_drag(
        &self,
        mut changes: &mut ChangesToBeMade,
        response: &egui::Response,
        display_size: egui::Vec2,
    ) -> bool {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let mouse_down_pos = vector!(pointer_pos[0], pointer_pos[1], 1.0);
            let tshirt_to_display = tshirt_to_display(self.central_viewport(display_size));
            changes += Box::new(move |app: &mut Self| {
                app.move_state
                    .event_mouse_down_movement(mouse_down_pos, tshirt_to_display);
            });
            true
        } else {
            changes += Box::new(move |app: &mut Self| {
                app.move_state.event_mouse_released();
            });
            false
        }
    }

    fn handle_central_movement_zoom(
        &self,
        mut changes: &mut ChangesToBeMade,
        ui: &egui::Ui,
        response: &egui::Response,
    ) -> bool {
        const ZOOM_RATE: f32 = 200.0;
        if response.hovered() {
            let zoom_delta_0 = 1.0 + ui.ctx().input(|i| i.smooth_scroll_delta)[1] / ZOOM_RATE;
            let zoom_delta_1 = ui.ctx().input(|i| i.zoom_delta());
            changes += Box::new(move |app: &mut Self| {
                app.move_state.handle_zoom(zoom_delta_0, zoom_delta_1);
            });
            zoom_delta_0 != 1.0 || zoom_delta_1 != 1.0
        } else {
            false
        }
    }

    fn paint_tshirt(&self, painter: &egui::Painter, display_size: egui::Vec2) {
        let tshirt_to_display = tshirt_to_display(self.central_viewport(display_size));

        let uv0 = egui::Pos2 { x: 0.0, y: 0.0 };
        let uv1 = egui::Pos2 { x: 1.0, y: 1.0 };

        let s0 = v3_to_egui(tshirt_to_display * dvector![0.0, 0.0, 1.0]);
        let s1 = v3_to_egui(tshirt_to_display * dvector![1.0, 1.0, 1.0]);

        let tshirt_art = self
            .tshirt_image_storage
            .tshirt_enum_to_image(self.selected_tshirt);

        painter.image(
            tshirt_art.id(),
            egui::Rect::from_min_max(s0, s1),
            egui::Rect::from_min_max(uv0, uv1),
            egui::Color32::WHITE,
        );
    }

    fn paint_artwork(&self, painter: &egui::Painter, display_size: egui::Vec2) {
        let tshirt_to_display = tshirt_to_display(self.central_viewport(display_size));
        let art_space_to_display = tshirt_to_display * self.art_space_to_shirt_matrix();
        let art_to_display = art_space_to_display * self.art_to_art_space_matrix();

        let a0 = v3_to_egui(art_to_display * dvector![0.0, 0.0, 1.0]);
        let a1 = v3_to_egui(art_to_display * dvector![1.0, 1.0, 1.0]);
        let uv0 = egui::Pos2 { x: 0.0, y: 0.0 };
        let uv1 = egui::Pos2 { x: 1.0, y: 1.0 };

        let cycle = self.selected_tool.get_cycles();
        let texture_to_display = if self
            .selected_tool
            .is_active(ReportTypes::PartialTransparency)
        {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            match cycle % 2 {
                0 => dependent_data.partial_transparency_problems.id(),
                _ => dependent_data.partial_transparency_fixed.id(),
            }
        } else if self.selected_tool.is_active(ReportTypes::Dpi) {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            dependent_data.partial_transparency_fixed.id()
        } else if self.selected_tool.is_active(ReportTypes::ThinLines) {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            match cycle % 2 {
                0 => dependent_data.thin_line_problems.id(),
                _ => self.get_selected_art().id(),
            }
        } else if self.selected_tool.is_active(ReportTypes::Bib) {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            match (cycle / 2) % 2 {
                0 => dependent_data.bib_opaque_mask.id(),
                _ => self.get_selected_art().id(),
            }
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

    fn paint_dpi_tool(&self, mut changes: &mut ChangesToBeMade, movement_happened: bool) {
        let dependent_data = self
            .art_storage
            .get_dependent_data(self.selected_art)
            .unwrap();
        let cycle = self.selected_tool.get_cycles() / 10;
        let slot = cycle % (dependent_data.dpi_top_hot_spots.len() as u32);
        let hot_spot = &dependent_data.dpi_top_hot_spots[slot as usize];
        let art_location = vector![hot_spot.location.x, hot_spot.location.y, 1.0];
        let art_to_tshirt = self.art_space_to_shirt_matrix() * self.art_to_art_space_matrix();
        let display_location = art_to_tshirt * art_location;

        if !movement_happened {
            changes += Box::new(move |app: &mut Self| {
                app.move_state.zoom = 10.0;
                app.move_state.target = display_location;
            });
        } else {
            changes += Box::new(move |app: &mut Self| {
                app.selected_tool.reset();
            });
        }
    }

    fn paint_area_used_tool(&self, painter: &egui::Painter, display_size: egui::Vec2) {
        const CYCLES_IN_AREA_USED_ANIMATION: u32 = 3;

        let tshirt_to_display = tshirt_to_display(self.central_viewport(display_size));
        let art_space_to_display = tshirt_to_display * self.art_space_to_shirt_matrix();

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

        // compute an offset in display units for a 3 cycle line animation
        let cycle = self.selected_tool.get_cycles() % CYCLES_IN_AREA_USED_ANIMATION;
        let percent_in_cycle: f32 = (cycle as f32) / (CYCLES_IN_AREA_USED_ANIMATION as f32);
        let dash_offset: f32 = percent_in_cycle * (dash_length + gap_length);

        let area_used_tool_dash_color = egui::Color32::from_rgb(200, 200, 200);
        let stroke = egui::Stroke::new(dash_width, area_used_tool_dash_color);

        painter.add(egui::Shape::dashed_line_with_offset(
            &art_space_border,
            stroke,
            &[dash_length],
            &[gap_length],
            dash_offset,
        ));
    }

    fn central_viewport(&self, display_size: egui::Vec2) -> ViewPort {
        ViewPort {
            zoom: self.move_state.zoom,
            target: self.move_state.target,
            display_size,
            tshirt_size: self.tshirt_image_storage.tshirt_image_size(),
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////
    //
    // Common Utiltiies
    //
    ////////////////////////////////////////////////////////////////////////////////////

    fn get_selected_art(&self) -> &LoadedImage {
        self.art_storage.get_art(self.selected_art)
    }

    fn get_selected_dependent_data(&self) -> Option<&ArtworkDependentData> {
        self.art_storage.get_dependent_data(self.selected_art)
    }

    fn is_report_ready(&self, report_type: ReportTypes) -> bool {
        let art = self.get_selected_art();
        let art_dependent_data = self.art_storage.get_dependent_data(self.selected_art);
        let report = self.report_templates.report_type_to_template(report_type);
        let metric = (report.generate_metric)(art, art_dependent_data);
        let status = (report.metric_to_status)(metric);
        status != ReportStatus::Unknown
    }

    fn are_all_reports_ready(&self) -> bool {
        self.is_report_ready(ReportTypes::Dpi)
            && self.is_report_ready(ReportTypes::AreaUsed)
            && self.is_report_ready(ReportTypes::Bib)
            && self.is_report_ready(ReportTypes::ThinLines)
            && self.is_report_ready(ReportTypes::PartialTransparency)
    }

    fn art_space_to_shirt_matrix(&self) -> Matrix3<f32> {
        art_space_to_tshirt(self.tshirt_image_storage.tshirt_image_size())
    }

    fn art_to_art_space_matrix(&self) -> Matrix3<f32> {
        let art = self.get_selected_art();
        art_to_art_space(art.size())
    }

    //////////////////////////////////////////////////////////////////
    //
    // All code that updates the application's state
    //
    //////////////////////////////////////////////////////////////////

    fn journal_changes_to_app_state(&mut self, changes: ChangesToBeMade) {
        for change in changes.changes.iter() {
            change(self);
        }
        self.recieve_asyncronous_data();
        self.icons.advance_cycle();
        self.notification_panel.update();
    }

    fn recieve_asyncronous_data(&mut self) {
        let data_attempt = self.async_data_to_app_receiver.try_recv();
        if data_attempt.is_ok() {
            let loaded_result = data_attempt.unwrap();
            match loaded_result {
                Err(e) => {
                    if e.id != ErrorTypes::FileImportAborted {
                        self.notification_panel.add_notice(e.msg());
                    }
                }
                Ok(f) => {
                    self.art_storage
                        .set_art(f.artwork, f.image, f.dependent_data);
                    self.selected_tool.reset();
                }
            }
        }
    }

    fn schedule_repaint_request_if_needed(&self, ctx: &egui::Context) {
        let mut time_to_repaint: u32 = u32::MAX;
        time_to_repaint = time_to_repaint.min(self.notification_panel.time_to_update());

        let display_loading_animation_instead_of_tools = self.are_all_reports_ready();
        if display_loading_animation_instead_of_tools {
            time_to_repaint = time_to_repaint.min(ICON_LOAD_ANIMATION_IN_MILLIS);
        }

        if self
            .selected_tool
            .is_active(ReportTypes::PartialTransparency)
            || self.selected_tool.is_active(ReportTypes::AreaUsed)
            || self.selected_tool.is_active(ReportTypes::ThinLines)
            || self.selected_tool.is_active(ReportTypes::Dpi)
            || self.selected_tool.is_active(ReportTypes::Bib)
        {
            time_to_repaint = time_to_repaint.min(self.selected_tool.time_to_next_epoch());
        }

        if time_to_repaint != u32::MAX {
            ctx.request_repaint_after(std::time::Duration::from_millis(time_to_repaint.into()))
        }
    }
}

fn mtexts(text: &String, scale: f32) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0 * scale)
}
