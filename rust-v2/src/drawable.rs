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

    /// Jeśli to focused input, zwraca pozycję kursora w content (w znakach)
    /// Layout użyje tego do obliczenia absolutnej pozycji kursora
    fn cursor_offset(&self) -> Option<usize> {
        None
    }
}
