use crate::drawable::{Display, Drawable, Wrap};
use crate::frame::Frame;
use crate::render_context::RenderContext;
use crate::span::Span;

#[derive(Clone, Debug, Default)]
pub struct Layout {
    margin: usize,
}

impl Layout {
    pub fn new() -> Self {
        Self { margin: 2 }
    }

    pub fn with_margin(mut self, margin: usize) -> Self {
        self.margin = margin;
        self
    }

    pub fn compose<'a, I>(&self, drawables: I, width: u16) -> RenderContext
    where
        I: IntoIterator<Item = &'a dyn Drawable>,
    {
        let mut ctx = LayoutContext::new(width as usize, self.margin);

        for drawable in drawables {
            ctx.place_drawable(drawable);
        }

        ctx.finish()
    }
}

struct LayoutContext {
    frame: Frame,
    width: usize,
    current_width: usize,
    current_line_index: usize,
    cursor_position: Option<(u16, usize)>,
}

impl LayoutContext {
    fn new(width: usize, margin: usize) -> Self {
        let width = width.saturating_sub(margin);
        let mut frame = Frame::new();
        frame.ensure_line();
        Self {
            frame,
            width,
            current_width: 0,
            current_line_index: 0,
            cursor_position: None,
        }
    }

    fn place_drawable(&mut self, drawable: &dyn Drawable) {
        // Block element zaczyna od nowej linii jeśli coś już jest w obecnej
        if drawable.display() == Display::Block && self.current_width > 0 {
            self.new_line();
        }

        // Sprawdź czy to focused input
        let cursor_offset = drawable.cursor_offset();
        let start_col = self.current_width;

        for span in drawable.spans() {
            self.place_span(span);
        }

        // Jeśli to focused input, oblicz pozycję kursora
        if let Some(offset) = cursor_offset {
            // Kursor jest na pozycji start_col + offset (w znakach unicode)
            let cursor_col = (start_col + offset) as u16;
            self.cursor_position = Some((cursor_col, self.current_line_index));
        }

        // Block element NIE kończy automatycznie linii
        // Następny element (Inline lub Block) sam zdecyduje czy chce być w tej samej linii
    }

    fn place_span(&mut self, span: Span) {
        if self.width == 0 || span.width() == 0 {
            return;
        }

        match span.wrap() {
            Wrap::No => self.place_no_wrap(span),
            Wrap::Yes => self.place_wrap(span),
        }
    }

    fn place_no_wrap(&mut self, span: Span) {
        let span_width = span.width();

        // Jeśli nie mieści się w aktualnej linii, przejdź do nowej
        if self.current_width > 0 && span_width > self.available_width() {
            self.new_line();
        }

        // Przytnij jeśli przekracza całą szerokość
        let (head, _) = if span_width > self.width {
            span.split_at_width(self.width)
        } else {
            (span, None)
        };

        self.push_span(head);
    }

    fn place_wrap(&mut self, mut span: Span) {
        while span.width() > 0 {
            if self.current_width >= self.width {
                self.new_line();
            }

            let available = self.available_width();
            if span.width() <= available {
                self.push_span(span);
                return;
            }

            let (head, tail) = span.split_at_width(available);
            if head.width() > 0 {
                self.push_span(head);
            }
            self.new_line();

            match tail {
                Some(rest) => span = rest,
                None => return,
            }
        }
    }

    fn push_span(&mut self, span: Span) {
        let w = span.width();
        self.frame.current_line_mut().push(span);
        self.current_width += w;
    }

    fn new_line(&mut self) {
        self.frame.new_line();
        self.current_width = 0;
        self.current_line_index += 1;
    }

    fn available_width(&self) -> usize {
        self.width.saturating_sub(self.current_width)
    }

    fn finish(mut self) -> RenderContext {
        self.frame.trim_trailing_empty();
        let mut ctx = RenderContext::new(self.frame);
        if let Some((col, line)) = self.cursor_position {
            ctx.set_cursor(col, line);
        }
        ctx
    }
}
