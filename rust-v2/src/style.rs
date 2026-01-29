use crossterm::style::Color;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
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
}
