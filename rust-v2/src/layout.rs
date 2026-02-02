use crate::frame::Frame;
use crate::node::Node;
use crate::span::{Span, Wrap};

#[derive(Clone, Debug, Default)]
pub struct Layout {
    margin: usize,
}

impl Layout {
    pub fn new() -> Self {
        Self { margin: 0 }
    }

    pub fn with_margin(mut self, margin: usize) -> Self {
        self.margin = margin;
        self
    }

    pub fn compose(&self, nodes: &[Node], width: u16) -> Frame {
        let mut ctx = LayoutContext::new(width as usize, self.margin);

        for node in nodes {
            ctx.place_node(node);
        }

        ctx.finish()
    }
}

struct LayoutContext {
    frame: Frame,
    width: usize,
    current_width: usize,
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
        }
    }

    fn place_node(&mut self, node: &Node) {
        for span in node.render() {
            if span.text() == "\n" {
                self.new_line();
                continue;
            }
            self.place_span(span);
        }
        self.new_line();
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
        if self.current_width > 0 && span_width > self.available_width() {
            self.new_line();
        }

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
    }

    fn available_width(&self) -> usize {
        self.width.saturating_sub(self.current_width)
    }

    fn finish(mut self) -> Frame {
        self.frame.trim_trailing_empty();
        self.frame
    }
}
