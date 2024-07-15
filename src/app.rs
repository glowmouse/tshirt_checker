extern crate nalgebra as na;
use crate::artwork::*;
use crate::async_tasks::*;
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
use na::{dvector, vector};

const TOOL_WIDTH: f32 = 20.0;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
pub struct TShirtCheckerApp {
    art_storage: ArtStorage,
    selected_art: Artwork,
    icons: IconStorage,
    move_state: MovementState,
    tshirt_storage: TShirtStorage,
    tshirt_selected_for: TShirtColors,
    report_templates: ReportTemplates,
    receiver: Receiver,
    sender: Sender,
    selected_tool: ToolSelection,
    animate_loading: bool,
    notice_panel: NoticePanel,
}

pub type AppEvent = Box<dyn Fn(&mut TShirtCheckerApp)>;

#[derive(Default)]
pub struct AppEvents {
    events: Vec<AppEvent>,
}

impl std::ops::AddAssign<AppEvent> for &mut AppEvents {
    fn add_assign(&mut self, rhs: AppEvent) {
        self.events.push(rhs);
    }
}

impl std::ops::AddAssign<AppEvent> for AppEvents {
    fn add_assign(&mut self, rhs: AppEvent) {
        self.events.push(rhs);
    }
}

impl TShirtCheckerApp {
    fn do_bottom_panel(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
            self.notice_panel.display(ui);
            powered_by_egui_and_eframe(ui);
        });
    }

    fn construct_viewport(&self, display_size: egui::Vec2) -> ViewPort {
        ViewPort {
            zoom: self.move_state.zoom,
            target: self.move_state.target,
            display_size,
            tshirt_size: self.tshirt_storage.size(),
        }
    }

    fn get_selected_art(&self) -> &LoadedImage {
        self.art_storage.get_art(self.selected_art)
    }

    fn handle_central_movement_drag(
        &self,
        mut new_events: &mut AppEvents,
        response: &egui::Response,
        display_size: egui::Vec2,
    ) -> bool {
        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let mouse_down_pos = vector!(pointer_pos[0], pointer_pos[1], 1.0);
            let tshirt_to_display = tshirt_to_display(self.construct_viewport(display_size));
            new_events += Box::new(move |app: &mut Self| {
                app.move_state
                    .event_mouse_down_movement(mouse_down_pos, tshirt_to_display);
            });
            true
        } else {
            new_events += Box::new(move |app: &mut Self| {
                app.move_state.event_mouse_released();
            });
            false
        }
    }

    fn handle_central_movement_zoom(
        &self,
        mut new_events: &mut AppEvents,
        ui: &egui::Ui,
        response: &egui::Response,
    ) -> bool {
        if response.hovered() {
            let zoom_delta_0 = 1.0 + ui.ctx().input(|i| i.smooth_scroll_delta)[1] / 200.0;
            let zoom_delta_1 = ui.ctx().input(|i| i.zoom_delta());
            new_events += Box::new(move |app: &mut Self| {
                app.move_state.handle_zoom(zoom_delta_0, zoom_delta_1);
            });
            zoom_delta_0 != 1.0 || zoom_delta_1 != 1.0
        } else {
            false
        }
    }

    fn handle_central_movement(
        &self,
        events: &mut AppEvents,
        ui: &egui::Ui,
        response: egui::Response,
        display_size: egui::Vec2,
    ) -> bool {
        let dragged = self.handle_central_movement_drag(events, &response, display_size);
        let zoomed = self.handle_central_movement_zoom(events, ui, &response);

        dragged || zoomed
    }

    fn paint_tshirt(&self, painter: &egui::Painter, display_size: egui::Vec2) {
        let tshirt_to_display = tshirt_to_display(self.construct_viewport(display_size));

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

    fn paint_artwork(&self, painter: &egui::Painter, display_size: egui::Vec2) {
        let tshirt_to_display = tshirt_to_display(self.construct_viewport(display_size));
        let art = self.get_selected_art();
        let art_space_to_display =
            tshirt_to_display * art_space_to_tshirt(self.tshirt_storage.size());
        let art_to_display = art_space_to_display * art_to_art_space(art.size());

        let a0 = v3_to_egui(art_to_display * dvector![0.0, 0.0, 1.0]);
        let a1 = v3_to_egui(art_to_display * dvector![1.0, 1.0, 1.0]);
        let uv0 = egui::Pos2 { x: 0.0, y: 0.0 };
        let uv1 = egui::Pos2 { x: 1.0, y: 1.0 };

        let cycle = self.selected_tool.get_cycles() % 2;
        let texture_to_display = if self
            .selected_tool
            .is_active(ReportTypes::PartialTransparency)
        {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            match cycle {
                0 => dependent_data.flagged_artwork.id(),
                _ => dependent_data.fixed_artwork.id(),
            }
        } else if self.selected_tool.is_active(ReportTypes::Dpi) {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            dependent_data.fixed_artwork.id()
        } else if self.selected_tool.is_active(ReportTypes::ThinLines) {
            let dependent_data = self
                .art_storage
                .get_dependent_data(self.selected_art)
                .unwrap();
            match cycle {
                0 => dependent_data.thin_lines.id(),
                _ => dependent_data.fixed_artwork.id(),
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

    fn do_dpi_tool(&self, mut new_events: &mut AppEvents, movement_happened: bool) {
        let dependent_data = self
            .art_storage
            .get_dependent_data(self.selected_art)
            .unwrap();
        let cycle = self.selected_tool.get_cycles() / 10;
        let slot = cycle % (dependent_data.top_hot_spots.len() as u32);
        let hot_spot = &dependent_data.top_hot_spots[slot as usize];
        let art_location = vector![hot_spot.location.x, hot_spot.location.y, 1.0];
        let art = self.get_selected_art();
        let art_to_tshirt =
            art_space_to_tshirt(self.tshirt_storage.size()) * art_to_art_space(art.size());
        let display_location = art_to_tshirt * art_location;

        // need to make modifications to self after dependent_data borrow is done.
        if !movement_happened {
            new_events += Box::new(move |app: &mut Self| {
                app.move_state.zoom = 10.0;
                app.move_state.target = display_location;
            });
        } else {
            // deselect tool if the user is trying to move or zoom.
            new_events += Box::new(move |app: &mut Self| {
                app.selected_tool.reset();
            });
        }
    }

    fn paint_area_used_tool(&self, painter: &egui::Painter, display_size: egui::Vec2) {
        let tshirt_to_display = tshirt_to_display(self.construct_viewport(display_size));
        let art_space_to_display =
            tshirt_to_display * art_space_to_tshirt(self.tshirt_storage.size());

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

    fn do_central_panel(&self, new_events: &mut AppEvents, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let display_size = ui.available_size_before_wrap();
            let (response, painter) =
                ui.allocate_painter(display_size, egui::Sense::click_and_drag());

            let movement_happened =
                self.handle_central_movement(new_events, ui, response, display_size);
            self.paint_tshirt(&painter, display_size);
            self.paint_artwork(&painter, display_size);

            if self.selected_tool.is_active(ReportTypes::Dpi) {
                self.do_dpi_tool(new_events, movement_happened);
            }
            if self.selected_tool.is_active(ReportTypes::AreaUsed) {
                self.paint_area_used_tool(&painter, display_size);
            }
        });
    }

    fn handle_tshirt_button(
        &self,
        mut new_events: &mut AppEvents,
        ui: &mut egui::Ui,
        scale: f32,
        color: TShirtColors,
    ) {
        let width = 80.0 * scale;
        let image: &LoadedImage = self.tshirt_storage.tshirt_enum_to_image(color);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(width);
        let is_selected = self.tshirt_selected_for == color;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            new_events += Box::new(move |app: &mut Self| {
                app.tshirt_selected_for = color;
            });
        }
    }

    fn handle_art_button(
        &self,
        mut new_events: &mut AppEvents,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
        artwork: Artwork,
    ) {
        let width = 80.0 * scale;
        let image: &LoadedImage = self.art_storage.get_art(artwork);
        let egui_image = egui::Image::from_texture(image.texture_handle()).max_width(width);
        let is_selected = self.selected_art == artwork;
        if ui
            .add(egui::widgets::ImageButton::new(egui_image).selected(is_selected))
            .clicked()
        {
            if self.art_storage.get_dependent_data(artwork).is_none() {
                cache_in_dependent_data(
                    ctx,
                    self.art_storage.get_art(artwork),
                    artwork,
                    &self.sender,
                );
            }
            new_events += Box::new(move |app: &mut Self| {
                app.selected_art = artwork;
                app.selected_tool.reset();
            });
        }
    }

    fn report_metric(
        &self,
        mut new_events: &mut AppEvents,
        ui: &mut egui::Ui,
        scale: f32,
        report_type: ReportTypes,
    ) {
        ui.horizontal(|ui| {
            StripBuilder::new(ui)
                .size(Size::exact(25.0 * scale))
                .size(Size::exact(150.0 * scale))
                .size(Size::exact(40.0 * scale))
                .size(Size::exact(20.0 * scale))
                .size(Size::exact(TOOL_WIDTH * scale))
                .horizontal(|mut strip| {
                    let art = self.get_selected_art();
                    let art_dependent_data = self.art_storage.get_dependent_data(self.selected_art);

                    let report = self.report_templates.report_type_to_template(report_type);
                    let metric = (report.generate_metric)(art, art_dependent_data);
                    let status = (report.metric_to_status)(metric);
                    if status == ReportStatus::Unknown {
                        new_events += Box::new(move |app: &mut Self| {
                            app.animate_loading = true;
                        })
                    }
                    let status_icon = self.icons.status_icon(status).max_width(25.0 * scale);
                    let tool_tip = report.tool_tip.clone();
                    let report_tip = report.report_tip.clone();

                    strip.cell(|ui| {
                        ui.add(status_icon).on_hover_text(&report_tip);
                    });
                    strip.cell(|ui| {
                        ui.label(mtexts(&report.label.to_string(), scale))
                            .on_hover_text(&report_tip);
                    });
                    strip.cell(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            let text = match metric {
                                Some(n) => format!("{}", n),
                                None => "???".to_string(),
                            };
                            ui.label(mtexts(&text, scale)).on_hover_text(&report_tip);
                        });
                    });
                    let cell_string = (if report.display_percent { "%" } else { "" }).to_string();
                    strip.cell(|ui| {
                        ui.label(mtexts(&cell_string, scale));
                    });
                    strip.cell(|ui| {
                        if status != ReportStatus::Pass && status != ReportStatus::Unknown {
                            let is_selected = self.selected_tool.is_active(report_type);
                            if ui
                                .add(
                                    self.icons
                                        .button(Icon::Tool, TOOL_WIDTH * scale)
                                        .selected(is_selected),
                                )
                                .on_hover_text(tool_tip)
                                .clicked()
                            {
                                new_events += Box::new(move |app: &mut Self| {
                                    app.selected_tool.set(report_type, !is_selected);
                                });
                            }
                        }
                    });
                });
        });
    }

    fn panel_separator(ui: &mut egui::Ui, scale: f32) {
        ui.add_space(5.0 * scale);
        ui.separator();
        ui.add_space(5.0 * scale);
    }

    fn display_title(ui: &mut egui::Ui, scale: f32) {
        Self::panel_separator(ui, scale);
        let width = 30.0 * scale;
        ui.vertical_centered(|ui| {
            ui.heading(egui::widget_text::RichText::from("T-Shirt Art Checker").size(width))
        });
        Self::panel_separator(ui, scale);
    }

    fn report_metrics(&self, new_events: &mut AppEvents, ui: &mut egui::Ui, scale: f32) {
        self.report_metric(new_events, ui, scale, ReportTypes::Dpi);
        self.report_metric(new_events, ui, scale, ReportTypes::AreaUsed);
        self.report_metric(new_events, ui, scale, ReportTypes::Bib);
        self.report_metric(new_events, ui, scale, ReportTypes::ThinLines);
        self.report_metric(new_events, ui, scale, ReportTypes::PartialTransparency);

        Self::panel_separator(ui, scale);
    }

    fn tshirt_selection_panel(&self, new_events: &mut AppEvents, ui: &mut egui::Ui, scale: f32) {
        ui.horizontal(|ui| {
            self.handle_tshirt_button(new_events, ui, scale, TShirtColors::Red);
            self.handle_tshirt_button(new_events, ui, scale, TShirtColors::Green);
            self.handle_tshirt_button(new_events, ui, scale, TShirtColors::Blue);
        });
        ui.horizontal(|ui| {
            self.handle_tshirt_button(new_events, ui, scale, TShirtColors::DRed);
            self.handle_tshirt_button(new_events, ui, scale, TShirtColors::DGreen);
            self.handle_tshirt_button(new_events, ui, scale, TShirtColors::DBlue);
        });
        Self::panel_separator(ui, scale);
    }

    fn artwork_selection_panel(
        &self,
        new_events: &mut AppEvents,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        scale: f32,
    ) {
        ui.horizontal(|ui| {
            self.handle_art_button(new_events, ui, ctx, scale, Artwork::Artwork0);
            self.handle_art_button(new_events, ui, ctx, scale, Artwork::Artwork1);
            self.handle_art_button(new_events, ui, ctx, scale, Artwork::Artwork2);
        });
        Self::panel_separator(ui, scale);
    }

    fn import_button(&self, ui: &mut egui::Ui, ctx: &egui::Context, scale: f32) {
        let width = 80.0 * scale;
        if ui
            .add(self.icons.button(Icon::Import, width))
            .on_hover_text("Import an image to the selected artwork slot.")
            .clicked()
        {
            do_load(ctx, self.selected_art, &self.sender);
        }
    }

    fn partial_transparency_fix_button(&self, ui: &mut egui::Ui, ctx: &egui::Context, scale: f32) {
        let width = 80.0 * scale;
        if ui
            .add(self.icons.button(Icon::FixPT, width))
            .on_hover_text(
                "Fix partial transparency problems by mapping all alpha values to 0 or 1.",
            )
            .clicked()
        {
            partialt_fix(
                ctx,
                self.art_storage.get_art(self.selected_art),
                self.selected_art,
                &self.sender,
            );
        }
    }

    fn do_right_panel(&self, new_events: &mut AppEvents, ctx: &egui::Context) {
        let screen = ctx.screen_rect();
        let size = screen.max - screen.min;
        let scale_x = (size.x * 0.33) / 260.0;
        let scale_y = (size.y - 150.0) / 650.0;
        let scale = scale_x.min(scale_y).clamp(0.20, 1.0);

        let targetx = 50.0 + 260.0 * scale;

        egui::SidePanel::right("stuff")
            .resizable(true)
            .min_width(targetx)
            .max_width(targetx)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    Self::display_title(ui, scale);
                    self.report_metrics(new_events, ui, scale);
                    self.tshirt_selection_panel(new_events, ui, scale);
                    self.artwork_selection_panel(new_events, ui, ctx, scale);

                    ui.horizontal(|ui| {
                        self.import_button(ui, ctx, scale);
                        self.partial_transparency_fix_button(ui, ctx, scale);
                    });
                })
            });
    }

    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<Payload>();
        let art_storage = ArtStorage::new(&cc.egui_ctx);
        let selected_art = Artwork::Artwork0;
        cache_in_dependent_data(
            &cc.egui_ctx,
            art_storage.get_art(selected_art),
            selected_art,
            &sender,
        );

        Self {
            art_storage,
            selected_art,
            tshirt_storage: TShirtStorage::new(&cc.egui_ctx),
            move_state: MovementState::new(),
            icons: IconStorage::new(&cc.egui_ctx),
            tshirt_selected_for: TShirtColors::Red,
            report_templates: ReportTemplates::new(),
            selected_tool: ToolSelection::new(),
            receiver,
            sender,
            animate_loading: false,
            notice_panel: NoticePanel::default(),
        }
    }
}

fn mtexts(text: &String, scale: f32) -> egui::widget_text::RichText {
    egui::widget_text::RichText::from(text).size(25.0 * scale)
}

impl eframe::App for TShirtCheckerApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut new_events: AppEvents = Default::default();
        let data_attempt = self.receiver.try_recv();
        if data_attempt.is_ok() {
            let loaded_result = data_attempt.unwrap();
            match loaded_result {
                Err(e) => {
                    if e.id != ErrorTypes::FileImportAborted {
                        new_events += Box::new(move |app: &mut Self| {
                            app.notice_panel.add_notice(e.msg());
                        });
                    }
                }
                Ok(f) => {
                    self.art_storage
                        .set_art(f.artwork, f.image, f.dependent_data);
                    self.selected_tool.reset();
                }
            }
        }
        self.do_bottom_panel(ctx);
        self.do_right_panel(&mut new_events, ctx);
        self.do_central_panel(&mut new_events, ctx);

        self.animate_loading = false;
        for closure in new_events.events.iter() {
            closure(self);
        }
        self.icons.advance_cycle();
        self.notice_panel.update();

        let mut time_to_repaint: u32 = u32::MAX;
        time_to_repaint = time_to_repaint.min(self.notice_panel.time_to_update());

        if self.animate_loading {
            time_to_repaint = time_to_repaint.min(ICON_LOAD_ANIMATION_IN_MILLIS);
        }

        if self
            .selected_tool
            .is_active(ReportTypes::PartialTransparency)
            || self.selected_tool.is_active(ReportTypes::AreaUsed)
            || self.selected_tool.is_active(ReportTypes::ThinLines)
            || self.selected_tool.is_active(ReportTypes::Dpi)
        {
            time_to_repaint = time_to_repaint.min(self.selected_tool.time_to_next_epoch());
        }

        if time_to_repaint != u32::MAX {
            ctx.request_repaint_after(std::time::Duration::from_millis(time_to_repaint.into()))
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
