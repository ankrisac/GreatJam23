use crate::nvec::*;

#[derive(Debug)]
struct Rect {
    a: Vec2<f32>,
    b: Vec2<f32>,
}
impl Rect {
    fn between(value: f32, a: f32, b: f32) -> bool {
        (a <= value && value <= b) || (b <= value && value <= a)
    }

    fn sized(origin: Vec2<f32>, size: Vec2<f32>) -> Self {
        Rect {
            a: origin,
            b: origin + size,
        }
    }
    fn contains(&self, pos: Vec2<f32>) -> bool {
        Self::between(pos.x, self.a.x, self.b.x) && Self::between(pos.y, self.a.y, self.b.y)
    }
}

pub struct Text {
    raw: String,
    id: String,
}
impl Text {
    pub fn with_id(value: &str, id: &str) -> Self {
        if !value.is_ascii() {
            eprintln!("Warning: Non-ascii text [{value}] not supported in gui");
        }
        Self {
            raw: value.to_owned(),
            id: id.to_owned(),
        }
    }

    fn get_id(&self) -> String {
        self.raw.clone() + &self.id
    }

    fn compute_size(&self, ui: &mut UserInterface) -> Vec2<f32> {
        let mut max_width: f32 = 0.0;
        let mut width: f32 = 0.0;
        let mut height: f32 = -ui.glyph_size.y;

        for byte in self.raw.as_bytes() {
            if *byte == b'\n' {
                max_width = max_width.max(width);
                width = 0.0;
                height -= ui.glyph_size.y;
            } else {
                width += ui.glyph_size.x;
            }
        }

        vec2(max_width.max(width), height)
    }
    fn paint(&self, ui: &mut UserInterface, color: Vec4<f32>) -> Vec2<f32> {
        let mut pos = ui.anchor;

        for byte in self.raw.as_bytes() {
            if *byte == b'\n' {
                pos.x = ui.anchor.x;
                pos.y -= ui.glyph_size.y;
            } else {
                ui.glyphs.push(crate::glyph::Glyph {
                    pos: vec3(pos.x, pos.y, 0.0),
                    codepoint: *byte as u32,
                    scale: ui.glyph_size,
                    color,
                });

                pos.x += ui.glyph_size.x;
            }
        }
        pos
    }
}
impl From<&str> for Text {
    fn from(value: &str) -> Self {
        if !value.is_ascii() {
            eprintln!("Warning: Non-ascii text [{value}] not supported in gui");
        }
        Self {
            raw: value.to_owned(),
            id: String::new(),
        }
    }
}

#[derive(Default)]
pub struct Response {
    pub clicked: bool,
    pub hover: bool,
    pub active: bool,
}

pub struct UserInterface {
    pub glyphs: Vec<crate::glyph::Glyph>,
    pub mouse: crate::input::MouseState,

    pub anchor: Vec2<f32>,

    pub glyph_unit: Vec2<f32>,
    pub glyph_size: Vec2<f32>,

    hot: String,
    active: String,
}

impl UserInterface {
    pub fn new() -> Self {
        let mut glyphs = Vec::new();
        glyphs.reserve(1024);

        Self {
            glyphs,
            mouse: crate::input::MouseState::default(),
            anchor: vec2(0.0, 0.0),
            glyph_unit: vec2(0.0, 0.0),
            glyph_size: vec2(0.0, 0.0),
            hot: String::new(),
            active: String::new(),
        }
    }

    pub fn set_fontsize(&mut self, size: f32) {
        self.glyph_size.x = size * self.glyph_unit.x;
        self.glyph_size.y = size * self.glyph_unit.y;
    }

    pub fn label(&mut self, text: impl Into<Text>) {
        let block: Text = text.into();
        self.anchor.y = block.paint(self, vec4(1.0, 1.0, 1.0, 1.0)).y - self.glyph_size.y;
    }
    pub fn button(&mut self, text: impl Into<Text>) -> Response {
        let block: Text = text.into();

        let id = block.get_id();
        let rect = Rect::sized(self.anchor, block.compute_size(self));


        let mut response = Response::default();
        if self.active == id {
            response.active = true;

            if self.mouse.released() {
                if self.hot == block.get_id() {
                    response.clicked = true;
                }
                self.active.clear();
            }
        } else if self.hot == id {
            if self.mouse.pressed() {
                response.active = true;

                self.active.clear();
                self.active.push_str(&id);
            }
        }

        if rect.contains(self.mouse.pos) {
            response.hover = true;
            self.hot.clear();
            self.hot.push_str(&id);
        }

        let mut color = vec4(1.0, 1.0, 1.0, 1.0);        
        if response.hover {
            color = vec4(0.2, 0.4, 0.8, 1.0);
        } else if response.active {
            color = vec4(0.9, 0.2, 0.3, 1.0);
        }

        self.anchor.y = block.paint(self, color).y - self.glyph_size.y;

        response
    }
}
