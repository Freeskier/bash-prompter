use crate::frame::Frame;

/// Kontekst renderowania zawierający Frame oraz informacje o kursorze
#[derive(Debug)]
pub struct RenderContext {
    /// Wyrenderowany frame
    pub frame: Frame,

    /// Pozycja kursora w frame (kolumna, linia) jeśli powinien być widoczny
    pub cursor_position: Option<(u16, usize)>,
}

impl RenderContext {
    pub fn new(frame: Frame) -> Self {
        Self {
            frame,
            cursor_position: None,
        }
    }

    pub fn with_cursor(mut self, col: u16, line: usize) -> Self {
        self.cursor_position = Some((col, line));
        self
    }

    pub fn set_cursor(&mut self, col: u16, line: usize) {
        self.cursor_position = Some((col, line));
    }
}
