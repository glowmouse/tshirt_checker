use crate::log::*;
use crate::time::*;
use std::rc::Rc;

pub type DisplayTimerPtr = Rc<dyn Time>;
pub type LogPtr = Rc<dyn AppLog>;

const NOTICE_TIME: u32 = 10000;
const FADE_TIME: u32 = 1024;
const FADE_AT: u32 = NOTICE_TIME - FADE_TIME;

pub struct NoticePanel {
    notifications: Vec<String>,
    display_timer: DisplayTimerPtr,
    recent_state_change: bool,
    currently_displaying: bool,
    reset_time: u64,
    log: LogPtr,
}

impl NoticePanel {
    pub fn new(timer: DisplayTimerPtr, log: LogPtr) -> Self {
        let notifications = Vec::new();
        let current_time = timer.ms_since_start();
        Self {
            notifications,
            display_timer: timer,
            recent_state_change: false,
            currently_displaying: false,
            reset_time: current_time,
            log,
        }
    }

    pub fn add_notice(&mut self, notice: impl Into<String>) {
        self.notifications.push(notice.into());
    }

    fn compute_alpha(&self) -> u8 {
        if !self.currently_displaying {
            255
        } else {
            let time_since_start = self.time_since_reset().min(NOTICE_TIME);
            let time_to_end = NOTICE_TIME - time_since_start;
            if time_since_start < FADE_TIME {
                (time_since_start * 255 / FADE_TIME).try_into().unwrap()
            } else if time_to_end < FADE_TIME {
                (time_to_end * 255 / FADE_TIME).try_into().unwrap()
            } else {
                255
            }
        }
    }

    pub fn display(&self, ui: &mut egui::Ui) {
        if self.currently_displaying {
            let label_text = &self.notifications[0];
            let alpha = self.compute_alpha();
            ui.horizontal(|ui| {
                let color = egui::Color32::from_rgba_premultiplied(255, 0, 0, alpha);
                ui.label(egui::widget_text::RichText::from(label_text).color(color));
            });
            self.log.log(format!("(NP {} {})", label_text, alpha))
        } else {
            self.log.log("(NP)".to_string());
        }
    }

    pub fn time_to_update(&self) -> u32 {
        if self.recent_state_change {
            100
        } else if !self.currently_displaying {
            u32::MAX
        } else {
            let alpha = self.compute_alpha();
            if alpha != 255 {
                100
            } else {
                let clamped_time_since_start = self.time_since_reset().min(FADE_AT);
                FADE_AT - clamped_time_since_start
            }
        }
    }

    fn reset_timer(&mut self) {
        self.reset_time = self.display_timer.ms_since_start();
    }

    fn time_since_reset(&self) -> u32 {
        (self.display_timer.ms_since_start() - self.reset_time)
            .try_into()
            .unwrap()
    }

    pub fn update(&mut self) {
        self.recent_state_change = false;
        if !self.notifications.is_empty() && !self.currently_displaying {
            self.reset_timer();
            self.recent_state_change = true;
            self.currently_displaying = true;
        }
        if self.currently_displaying && self.time_since_reset() > NOTICE_TIME {
            self.notifications.remove(0);
            self.recent_state_change = true;
            self.currently_displaying = false;
        }
    }
}

#[cfg(test)]
mod notice_panel_should {
    use super::*;

    pub fn _create_test_context() -> egui::Context {
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::empty()); // prevent fonts from being loaded (save CPU time)
        ctx
    }

    pub fn _run_code_with_context(
        ctx: &mut egui::Context,
        mut add_contents: impl FnMut(&mut egui::Ui),
    ) {
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                add_contents(ui);
            });
        });
    }

    #[test]
    fn fade_in_when_notification_occurs() {
        // Test Setup
        let mut ctx = _create_test_context();
        let fake_time = Rc::new(FakeTime::default());
        let string_log = Rc::new(StringLog::default());
        let mut notice_panel: NoticePanel = NoticePanel::new(fake_time.clone(), string_log.clone());

        // Run display with initial state
        _run_code_with_context(&mut ctx, |ui| notice_panel.display(ui));

        // Advance time then add a notice.  Run display again
        fake_time.advance(10);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.add_notice("T0");
            notice_panel.add_notice("T1");
            notice_panel.update();
            notice_panel.display(ui);
        });

        // Advance time a bit more and do update/ display again
        fake_time.advance(512);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        // Advance time a bit more and do update/ display again
        fake_time.advance(512);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        let fade_in_should_start_at = 10000 - 1024 - 1024;
        fake_time.advance(fade_in_should_start_at);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        fake_time.advance(512);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        fake_time.advance(512);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        fake_time.advance(1);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        fake_time.advance(1);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        fake_time.advance(512);
        _run_code_with_context(&mut ctx, |ui| {
            notice_panel.update();
            notice_panel.display(ui);
        });
        assert_eq!("(NP) (NP T0 0) (NP T0 127) (NP T0 255) (NP T0 255) (NP T0 127) (NP T0 0) (NP) (NP T1 0) (NP T1 127) ", string_log._get_all());
    }
}
