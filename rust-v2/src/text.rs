use crate::drawable::{Display, Drawable, DrawableOptions, Wrap};
use crate::span::Span;
use crate::style::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    text: String,
    display: Display,
    options: DrawableOptions,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            display: Display::Inline,
            options: DrawableOptions::default(),
        }
    }

    pub fn with_display(mut self, display: Display) -> Self {
        self.display = display;
        self
    }

    pub fn with_options(mut self, options: DrawableOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.options = self.options.with_style(style);
        self
    }

    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.options = self.options.with_wrap(wrap);
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
                .with_style(self.options.style().clone())
                .with_wrap(self.options.wrap()),
        ]
    }

    fn display(&self) -> Display {
        self.display
    }

    fn options(&self) -> &DrawableOptions {
        &self.options
    }
}
