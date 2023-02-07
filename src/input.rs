use crate::nvec::Vec2;

use winit::event::{ElementState, MouseButton};

#[derive(Clone, Copy, Debug, Default)]
pub struct ButtonState {
    curr: bool,
    prev: bool
}
impl ButtonState {
    pub fn set(&mut self, state: ElementState) {
        self.prev = self.curr;
        match state {
            ElementState::Pressed => self.curr = true,
            ElementState::Released => self.curr = false,
        }
    }
    pub fn refresh(&mut self) {
        self.prev = self.curr;
    }

    pub fn released(&self) -> bool {
        !self.curr && self.prev
    }
    pub fn pressed(&self) -> bool {
        self.curr && !self.prev 
    }
}


#[derive(Clone, Debug, Default)]
pub struct MouseState {
    pub pos: Vec2<f32>,
    pub delta: Vec2<f32>,

    pub left: ButtonState,
    pub right: ButtonState,
    pub middle: ButtonState,
}

impl MouseState {
    pub fn set_pos(&mut self, new_pos: Vec2<f32>) {
        self.delta = new_pos - self.pos;
        self.pos = new_pos;
    }
    pub fn set_state(&mut self, new_state: ElementState, button: winit::event::MouseButton) {
        match button {
            MouseButton::Left => self.left.set(new_state),
            MouseButton::Right => self.right.set(new_state),
            MouseButton::Middle => self.middle.set(new_state),
            _ => {}
        }
    }

    pub fn refresh(&mut self) {
        self.left.refresh();
        self.right.refresh();
        self.middle.refresh();
    }

    pub fn pressed(&self) -> bool {
        self.left.pressed() || self.right.pressed() || self.middle.pressed()
    }
    pub fn released(&self) -> bool {
        self.left.released() || self.right.released() || self.middle.released()
    }
}

pub struct Input {
    pub mouse: MouseState,
}

impl Input {
    pub fn new() -> Self {
        let mouse = MouseState::default();
        Self { mouse }
    }
}
