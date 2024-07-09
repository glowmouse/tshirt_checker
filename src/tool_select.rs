use crate::report_templates::*;
use web_time::SystemTime;

const TOOL_TOGGLE_RATE: u32 = 500; // in ms

pub struct ToolSelection {
    tool_selected_at: SystemTime,
    tool_selected_for: std::option::Option<ReportTypes>,
}

impl ToolSelection {
    pub fn new() -> Self {
        Self {
            tool_selected_for: None,
            tool_selected_at: SystemTime::now(),
        }
    }
    pub fn time_since_selection(&self) -> u32 {
        self.tool_selected_at
            .elapsed()
            .unwrap()
            .as_millis()
            .try_into()
            .unwrap()
    }
    pub fn reset(&mut self) {
        self.tool_selected_for = None;
    }
    pub fn set(&mut self, tool: ReportTypes, active: bool) {
        if active {
            self.tool_selected_for = Some(tool);
            self.tool_selected_at = SystemTime::now();
        } else {
            self.reset();
        }
    }
    pub fn get_cycles(&self) -> u32 {
        self.time_since_selection() / TOOL_TOGGLE_RATE
    }
    pub fn is_active(&self, report_type: ReportTypes) -> bool {
        self.tool_selected_for.is_some() && self.tool_selected_for.unwrap() == report_type
    }

    pub fn time_to_next_epoch(&self) -> u32 {
        let time_in_ms = self.time_since_selection();
        let next_epoch = (time_in_ms / TOOL_TOGGLE_RATE + 1) * TOOL_TOGGLE_RATE + 1;
        next_epoch - time_in_ms
    }
}
