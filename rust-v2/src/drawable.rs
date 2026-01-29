use crate::span::Span;

use crate::style::Style;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Inline,
    Block,
    Sticky,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wrap {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawableOptions {
    style: Style,
    wrap: Wrap,
}

impl Default for DrawableOptions {
    fn default() -> Self {
        Self {
            style: Style::default(),
            wrap: Wrap::Yes,
        }
    }
}

impl DrawableOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn wrap(&self) -> Wrap {
        self.wrap
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }
}

pub trait Drawable {
    fn spans(&self) -> Vec<Span>;
    fn display(&self) -> Display;
    fn options(&self) -> &DrawableOptions;
}
