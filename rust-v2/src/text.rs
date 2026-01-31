use crate::drawable::{Display, Drawable, Wrap};
use crate::span::Span;
use crate::style::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    text: String,
    display: Display,
    style: Style,
    wrap: Wrap,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            display: Display::Inline,
            style: Style::default(),
            wrap: Wrap::Yes,
        }
    }

    pub fn with_display(mut self, display: Display) -> Self {
        self.display = display;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

impl Drawable for Text {
    fn spans(&self) -> Vec<Span> {
        vec![
            Span::new(self.text.clone())
                .with_style(self.style.clone())
                .with_wrap(self.wrap),
        ]
    }

    fn display(&self) -> Display {
        self.display
    }
}
