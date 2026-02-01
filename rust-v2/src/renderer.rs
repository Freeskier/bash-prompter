use crate::render_context::RenderContext;
use crossterm::{cursor, queue, terminal};
use std::io::{Result, Write};

pub struct Renderer {
    anchor_row: Option<u16>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { anchor_row: None }
    }

    pub fn render<W: Write>(&mut self, context: &RenderContext, out: &mut W) -> Result<()> {
        let anchor = self.get_or_init_anchor()?;
        let new_anchor = self.clear_and_render(context, anchor, out)?;
        // Aktualizuj anchor jeśli się zmienił (np. po scrollowaniu)
        self.anchor_row = Some(new_anchor);
        Ok(())
    }

    pub fn render_at<W: Write>(
        &mut self,
        context: &RenderContext,
        anchor: u16,
        out: &mut W,
    ) -> Result<()> {
        let new_anchor = self.clear_and_render(context, anchor, out)?;
        self.anchor_row = Some(new_anchor);
        Ok(())
    }

    fn get_or_init_anchor(&mut self) -> Result<u16> {
        if let Some(anchor) = self.anchor_row {
            Ok(anchor)
        } else {
            let (_, row) = cursor::position()?;
            self.anchor_row = Some(row);
            Ok(row)
        }
    }

    fn clear_and_render<W: Write>(
        &self,
        context: &RenderContext,
        mut anchor: u16,
        out: &mut W,
    ) -> Result<u16> {
        // Ukryj kursor podczas renderowania
        queue!(out, cursor::Hide)?;

        // Sprawdź czy potrzebujemy przewinąć terminal
        let (_, terminal_height) = terminal::size()?;
        let needed_lines = context.frame.lines().len() as u16;
        let end_row = anchor.saturating_add(needed_lines);

        // Jeśli wyjdziemy poza terminal, musimy stworzyć miejsce
        if end_row > terminal_height {
            let overflow = end_row - terminal_height;

            // Idź na koniec terminala i dodaj nowe linie (to przewinie zawartość w górę)
            queue!(out, cursor::MoveTo(0, terminal_height - 1))?;
            for _ in 0..overflow {
                write!(out, "\n")?;
            }
            out.flush()?;

            // Po przewinięciu anchor przesunął się w górę o `overflow` linii
            anchor = anchor.saturating_sub(overflow);
        }

        queue!(out, cursor::MoveTo(0, anchor))?;
        queue!(out, terminal::Clear(terminal::ClearType::FromCursorDown))?;

        // Renderuj frame
        for (idx, line) in context.frame.lines().iter().enumerate() {
            let row = anchor + idx as u16;
            queue!(out, cursor::MoveTo(0, row))?;
            write!(out, "{}", line)?;
        }

        // Ustaw kursor jeśli jest w kontekście
        if let Some((col, line)) = context.cursor_position {
            let row = anchor + line as u16;
            queue!(out, cursor::MoveTo(col, row))?;
            queue!(out, cursor::Show)?;
        } else {
            // Jeśli nie ma kursora, ustaw na anchor i ukryj
            queue!(out, cursor::MoveTo(0, anchor))?;
        }

        out.flush()?;
        Ok(anchor) // Zwróć nowy anchor
    }

    pub fn anchor_row(&self) -> Option<u16> {
        self.anchor_row
    }

    /// Pozycja kursora po ostatniej linii frame'a
    pub fn end_row(&self, context: &RenderContext) -> Option<u16> {
        self.anchor_row
            .map(|anchor| anchor + context.frame.lines().len() as u16)
    }

    /// Przesuń kursor na koniec wyrenderowanego contentu
    pub fn move_to_end<W: Write>(&self, context: &RenderContext, out: &mut W) -> Result<()> {
        if let Some(end) = self.end_row(context) {
            queue!(out, cursor::MoveTo(0, end))?;
            out.flush()?;
        }
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
