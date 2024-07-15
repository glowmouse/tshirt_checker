use web_time::SystemTime;
const NOTICE_TIME: u32 = 10000;

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

    pub fn display(&self, ui: &mut egui::Ui) {
        let label_text = if !self.notifications.is_empty() {
            &self.notifications[0]
        } else {
            ""
        };
        ui.horizontal(|ui| {
            ui.label(egui::widget_text::RichText::from(label_text).color(egui::Color32::RED));
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
            200
        } else if self.display_start.is_none() {
            u32::MAX
        } else {
            let clamped_time_since_start = self.time_since_start().min(NOTICE_TIME);
            NOTICE_TIME - clamped_time_since_start + 200
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
