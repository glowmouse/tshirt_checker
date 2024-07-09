extern crate nalgebra as na;
use na::{vector, Matrix3, Vector3};

pub struct MovementState {
    pub zoom: f32,
    pub target: Vector3<f32>,
    last_tshirt_pos: std::option::Option<Vector3<f32>>,
    mouse_to_tshirt_at_mouse_down_event: std::option::Option<Matrix3<f32>>,
}

impl MovementState {
    pub fn new() -> Self {
        Self {
            zoom: 1.0,
            target: vector![0.50, 0.50, 1.0],
            last_tshirt_pos: None,
            mouse_to_tshirt_at_mouse_down_event: None,
        }
    }
    pub fn event_mouse_released(&mut self) {
        self.last_tshirt_pos = None;
        self.mouse_to_tshirt_at_mouse_down_event = None;
        assert!(!self.is_currently_dragging());
    }

    pub fn event_mouse_down_movement(
        &mut self,
        mouse_pos: Vector3<f32>,
        tshirt_to_display: Matrix3<f32>,
    ) {
        if !self.is_currently_dragging() {
            self.start_dragging(mouse_pos, tshirt_to_display);
        } else {
            self.continue_dragging(mouse_pos);
        }
        assert!(self.is_currently_dragging());
    }

    pub fn handle_zoom(&mut self, zoom_delta_0: f32, zoom_delta_1: f32) {
        let zoom_delta = if zoom_delta_0 != 1.0 {
            zoom_delta_0
        } else {
            zoom_delta_1
        };

        self.zoom *= zoom_delta;
        if self.zoom < 1.0 {
            self.zoom = 1.0;
        }
    }

    fn is_currently_dragging(&self) -> bool {
        self.last_tshirt_pos.is_some()
    }

    fn start_dragging(&mut self, mouse_pos: Vector3<f32>, tshirt_to_display: Matrix3<f32>) {
        assert!(!self.is_currently_dragging());
        self.mouse_to_tshirt_at_mouse_down_event = Some(tshirt_to_display.try_inverse().unwrap());
        self.last_tshirt_pos = Some(self.mouse_to_tshirt_pos(mouse_pos));
        assert!(self.is_currently_dragging());
    }

    fn continue_dragging(&mut self, mouse_pos: Vector3<f32>) {
        assert!(self.is_currently_dragging());
        let tshirt_position = self.mouse_to_tshirt_pos(mouse_pos);
        let last_tshirt_pos = self.last_tshirt_pos.unwrap();

        self.target = self.target + last_tshirt_pos - tshirt_position;
        self.last_tshirt_pos = Some(tshirt_position);
        assert!(self.is_currently_dragging());
    }

    fn mouse_to_tshirt_pos(&self, mouse_pos: Vector3<f32>) -> Vector3<f32> {
        let mouse_to_tshirt = self.mouse_to_tshirt_at_mouse_down_event.unwrap();
        mouse_to_tshirt * mouse_pos
    }
}
