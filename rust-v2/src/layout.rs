use crate::drawable::{Display, Drawable, Wrap};
use crate::frame::Frame;
use crate::span::Span;

#[derive(Clone, Debug, Default)]
pub struct Layout;

impl Layout {
    pub fn new() -> Self {
        Self
    }

    pub fn compose<'a, I>(&self, drawables: I, width: u16) -> Frame
    where
        I: IntoIterator<Item = &'a dyn Drawable>,
    {
        let width = width.saturating_sub(2) as usize;
        let mut frame = Frame::new();
        frame.ensure_line();
        let mut current_width = 0usize;

        for drawable in drawables {
            if drawable.display() == Display::Block && current_width != 0 {
                frame.new_line();
                current_width = 0;
            }

            for span in drawable.spans() {
                self.place_span(&mut frame, &mut current_width, span, width);
            }

            if drawable.display() == Display::Block && current_width != 0 {
                frame.new_line();
                current_width = 0;
            }
        }

        frame.trim_trailing_empty();
        frame
    }

    fn place_span(&self, frame: &mut Frame, current_width: &mut usize, span: Span, width: usize) {
        if width == 0 || span.width() == 0 {
            return;
        }

        match span.wrap() {
            Wrap::No => self.place_no_wrap(frame, current_width, span, width),
            Wrap::Yes => self.place_wrap(frame, current_width, span, width),
        }
    }

    fn place_no_wrap(
        &self,
        frame: &mut Frame,
        current_width: &mut usize,
        span: Span,
        width: usize,
    ) {
        let span_width = span.width();
        if *current_width != 0 && span_width > width - *current_width {
            frame.new_line();
            *current_width = 0;
        }

        let (head, _) = if span_width > width {
            span.split_at_width(width)
        } else {
            (span, None)
        };

        frame.current_line_mut().push(head);
        *current_width = (*current_width + span_width).min(width);
    }

    fn place_wrap(
        &self,
        frame: &mut Frame,
        current_width: &mut usize,
        mut span: Span,
        width: usize,
    ) {
        loop {
            if *current_width == width {
                frame.new_line();
                *current_width = 0;
            }

            let available = width - *current_width;
            if span.width() <= available {
                let span_width = span.width();
                frame.current_line_mut().push(span);
                *current_width += span_width;
                return;
            }

            let (head, tail) = span.split_at_width(available);
            if head.width() > 0 {
                frame.current_line_mut().push(head);
            }
            frame.new_line();
            *current_width = 0;

            if let Some(next) = tail {
                span = next;
            } else {
                return;
            }
        }
    }
}
