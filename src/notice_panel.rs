use web_time::SystemTime;
const NOTICE_TIME: u32 = 10000;
const FADE_TIME: u32 = 1024;
const FADE_AT: u32 = NOTICE_TIME - FADE_TIME;

#[derive(Default)]
pub struct NoticePanel {
    notifications: Vec<String>,
    display_start: Option<SystemTime>,
    recent_state_change: bool,
}

impl NoticePanel {
    pub fn add_notice(&mut self, notice: String) {
        self.notifications.push(notice);
    }

    fn compute_alpha(&self) -> u8 {
        if self.display_start.is_none() {
            255
        } else {
            let time_since_start = self.time_since_start().min(NOTICE_TIME);
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

    fn time_since_start(&self) -> u32 {
        let display_start = self.display_start.unwrap();
        let time_since_start: u32 = display_start
            .elapsed()
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap();
        time_since_start
    }

    pub fn time_to_update(&self) -> u32 {
        if self.recent_state_change {
            100
        } else if self.display_start.is_none() {
            u32::MAX
        } else {
            let alpha = self.compute_alpha();
            if alpha != 255 {
                100
            } else {
                let clamped_time_since_start = self.time_since_start().min(FADE_AT);
                FADE_AT - clamped_time_since_start
            }
        }
    }

    pub fn update(&mut self) {
        self.recent_state_change = false;
        if !self.notifications.is_empty() && self.display_start.is_none() {
            self.display_start = Some(SystemTime::now());
            self.recent_state_change = true;
        }
        if self.display_start.is_some() {
            let time_since_start = self.time_since_start();
            if time_since_start > NOTICE_TIME {
                self.display_start = None;
                self.notifications.remove(0);
                self.recent_state_change = true;
            }
        }
    }
}
