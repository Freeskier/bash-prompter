use crate::span::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Inline,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Wrap {
    #[default]
    Yes,
    No,
}

pub trait Drawable {
    fn spans(&self) -> Vec<Span>;
    fn display(&self) -> Display;
}
