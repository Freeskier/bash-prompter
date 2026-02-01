use crossterm::style::{Attribute, Color};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    attributes: Vec<Attribute>,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fg(&self) -> Option<Color> {
        self.fg
    }

    pub fn bg(&self) -> Option<Color> {
        self.bg
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn with_fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn with_colors(mut self, fg: Color, bg: Color) -> Self {
        self.fg = Some(fg);
        self.bg = Some(bg);
        self
    }

    pub fn with_attribute(mut self, attribute: Attribute) -> Self {
        self.attributes.push(attribute);
        self
    }

    pub fn with_attributes(mut self, attributes: Vec<Attribute>) -> Self {
        self.attributes = attributes;
        self
    }
}
