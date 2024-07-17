use crate::time::*;
//use std::rc::Rc;

pub type DisplayTimerPtr = Box<dyn Time>;

const NOTICE_TIME: u32 = 10000;
const FADE_TIME: u32 = 1024;
const FADE_AT: u32 = NOTICE_TIME - FADE_TIME;

pub struct NoticePanel {
    notifications: Vec<String>,
    display_timer: DisplayTimerPtr,
    recent_state_change: bool,
    currently_displaying: bool,
    reset_time: u64,
}

impl NoticePanel {
    pub fn new(timer: DisplayTimerPtr) -> Self {
        let notifications = Vec::new();
        let current_time = timer.ms_since_start();
        Self {
            notifications,
            display_timer: timer,
            recent_state_change: false,
            currently_displaying: false,
            reset_time: current_time,
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
        let label_text = if !self.notifications.is_empty() {
            &self.notifications[0]
        } else {
            ""
        };
        ui.horizontal(|ui| {
            let color = egui::Color32::from_rgba_premultiplied(255, 0, 0, self.compute_alpha());
            ui.label(egui::widget_text::RichText::from(label_text).color(color));
        });
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

    #[test]
    fn fade_in_when_notification_occurs() {
        let fake_time = Box::new(FakeTime::default());
        let mut notice_panel: NoticePanel = NoticePanel::new(fake_time);
        notice_panel.add_notice("Testing");
        notice_panel.update();
        // Yeah, smart pointers are a learning project.
        //fake_time.advance(10);
    }
}
