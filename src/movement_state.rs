extern crate nalgebra as na;
use na::{vector, Matrix3, Vector3};

pub struct MovementState {
    pub zoom: f32,
    pub target: Vector3<f32>,
    last_drag_pos: std::option::Option<Vector3<f32>>,
    drag_display_to_tshirt: std::option::Option<Matrix3<f32>>,
}

impl MovementState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            target: vector![0.50, 0.50, 1.0],
            last_drag_pos: None,
            drag_display_to_tshirt: None,
        }
    }
    pub fn event_mouse_released(&mut self) {
        self.last_drag_pos = None;
        self.drag_display_to_tshirt = None;
    }

    fn is_currently_dragging(&self) -> bool {
        self.last_drag_pos.is_some()
    }

    fn start_dragging(&mut self, current_drag_pos: Vector3<f32>, tshirt_to_display: Matrix3<f32>) {
        self.drag_display_to_tshirt = Some(tshirt_to_display.try_inverse().unwrap());
        self.last_drag_pos = Some(current_drag_pos);
    }

    fn continue_dragging(&mut self, current_drag_pos: Vector3<f32>) {
        let last_drag_pos = self.last_drag_pos.unwrap();
        let display_to_artspace = self.drag_display_to_tshirt.unwrap();
        let last = display_to_artspace * last_drag_pos;
        let curr = display_to_artspace * current_drag_pos;
        self.target = self.target + last - curr;
        self.last_drag_pos = Some(current_drag_pos);
    }

    pub fn event_mouse_down_movement(
        &mut self,
        current_drag_pos: Vector3<f32>,
        tshirt_to_display: Matrix3<f32>,
    ) {
        if self.is_currently_dragging() {
            self.continue_dragging(current_drag_pos);
        } else {
            self.start_dragging(current_drag_pos, tshirt_to_display);
        }
    }

    pub fn handle_zoom(&mut self, zoom_delta_0: f32, zoom_delta_1: f32) -> bool {
        let zoom_delta = if zoom_delta_0 != 1.0 {
            zoom_delta_0
        } else {
            zoom_delta_1
        };

        self.zoom *= zoom_delta;
        if self.zoom < 1.0 {
            self.zoom = 1.0;
        }
        zoom_delta != 1.0
    }
}
