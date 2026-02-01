use crate::input::Input;
use crate::span::Span;
use crate::style::Style;
use crossterm::style::Color;
use unicode_width::UnicodeWidthStr;

pub enum Node {
    Text(String),
    Input(Box<dyn Input>),
}

impl Node {
    pub fn text(text: impl Into<String>) -> Self {
        Node::Text(text.into())
    }

    pub fn input(input: impl Input + 'static) -> Self {
        Node::Input(Box::new(input))
    }

    pub fn as_input(&self) -> Option<&dyn Input> {
        match self {
            Node::Input(input) => Some(input.as_ref()),
            _ => None,
        }
    }

    pub fn as_input_mut(&mut self) -> Option<&mut dyn Input> {
        match self {
            Node::Input(input) => Some(input.as_mut()),
            _ => None,
        }
    }

    pub fn render(&self) -> Vec<Span> {
        match self {
            Node::Text(text) => vec![Span::new(text.clone())],
            Node::Input(input) => {
                let mut spans = vec![Span::new(input.label()), Span::new(": ")];

                if input.is_focused() {
                    spans.push(Span::new("["));

                    if let Some(err) = input.error() {
                        let error_style = Style::new()
                            .with_fg(Color::Red)
                            .with_attribute(crossterm::style::Attribute::Bold);
                        spans.push(Span::new("✗ ").with_style(error_style.clone()));
                        spans.push(Span::new(err).with_style(error_style));
                    } else {
                        let content_spans = input.render_content();
                        let content_width: usize =
                            content_spans.iter().map(|s| s.text().width()).sum();
                        spans.extend(content_spans);

                        if content_width < input.min_width() {
                            let padding = input.min_width() - content_width;
                            spans.push(Span::new(" ".repeat(padding)));
                        }
                    }

                    spans.push(Span::new("]"));
                } else {
                    spans.extend(input.render_content());
                }

                spans
            }
        }
    }

    pub fn cursor_offset(&self) -> Option<usize> {
        match self {
            Node::Input(input) if input.is_focused() => {
                let label_len = input.label().width() + 2;
                let bracket_len = 1;
                let content_offset = input.cursor_offset_in_content();
                Some(label_len + bracket_len + content_offset)
            }
            _ => None,
        }
    }
}
