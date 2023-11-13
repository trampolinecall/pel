// TODO: REMOVE this module (these are temporary replacements for the sfml structs while i figure out how everything is supposed to work)
// graphics utilities

pub(crate) struct GraphicsContext {
    pub(crate) fonts: Fonts,
}

#[derive(Copy, Clone, PartialEq)]
pub(crate) struct Vector2f {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

impl Vector2f {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub(crate) struct Fonts {
    pub(crate) text_font: Font,
    pub(crate) monospace_font: Font,
}
pub(crate) enum Font {
    Text,
    Monospace,
}

impl Fonts {
    pub(crate) fn text_font(&self) -> &Font {
        &self.text_font
    }

    pub(crate) fn monospace_font(&self) -> &Font {
        &self.monospace_font
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Color {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8,
    pub(crate) a: u8,
}

impl Color {
    pub(crate) fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub(crate) fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub(crate) fn to_css_color(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

// this isnt even related to graphics!
// TODO: REMOVE this too
pub(crate) enum Key {
    Space,
}
