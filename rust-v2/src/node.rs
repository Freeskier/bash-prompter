use crate::input::Input;
use crate::span::Span;
use crate::theme::Theme;
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

    pub fn render(&self, inline_error_message: bool, theme: &Theme) -> Vec<Span> {
        match self {
            Node::Text(text) => vec![Span::new(text.clone())],
            Node::Input(input) => {
                let mut spans = vec![Span::new(input.label()), Span::new(": ")];
                let error_style = theme.error.clone();

                if input.is_focused() {
                    spans.push(Span::new("["));
                    let content_spans = if inline_error_message {
                        if let Some(err) = input.error() {
                            vec![
                                Span::new("✗ ").with_style(error_style.clone()),
                                Span::new(err).with_style(error_style.clone()),
                            ]
                        } else {
                            input.render_content()
                        }
                    } else {
                        let mut spans = input.render_content();
                        if input.error().is_some() {
                            spans = spans
                                .into_iter()
                                .map(|span| span.with_style(error_style.clone()))
                                .collect();
                        }
                        spans
                    };

                    let content_width: usize =
                        content_spans.iter().map(|s| s.text().width()).sum();
                    spans.extend(content_spans);

                    if content_width < input.min_width() {
                        let padding = input.min_width() - content_width;
                        spans.push(Span::new(" ".repeat(padding)));
                    }

                    spans.push(Span::new("]"));
                } else {
                    let content_spans = if inline_error_message {
                        if let Some(err) = input.error() {
                            vec![
                                Span::new("✗ ").with_style(error_style.clone()),
                                Span::new(err).with_style(error_style.clone()),
                            ]
                        } else {
                            input.render_content()
                        }
                    } else {
                        let mut spans = input.render_content();
                        if input.error().is_some() {
                            spans = spans
                                .into_iter()
                                .map(|span| span.with_style(error_style.clone()))
                                .collect();
                        }
                        spans
                    };
                    spans.extend(content_spans);
                }

                spans
            }
        }
    }

    pub fn render_field(&self, inline_error_message: bool, theme: &Theme) -> Vec<Span> {
        match self {
            Node::Text(text) => vec![Span::new(text.clone())],
            Node::Input(input) => {
                let mut spans = Vec::new();
                let error_style = theme.error.clone();

                spans.push(Span::new("["));
                let content_spans = if inline_error_message {
                    if let Some(err) = input.error() {
                        vec![
                            Span::new("✗ ").with_style(error_style.clone()),
                            Span::new(err).with_style(error_style.clone()),
                        ]
                    } else {
                        input.render_content()
                    }
                } else {
                    let mut spans = input.render_content();
                    if input.error().is_some() {
                        spans = spans
                            .into_iter()
                            .map(|span| {
                                let merged = span.style().clone().merge(&error_style);
                                span.with_style(merged)
                            })
                            .collect();
                    }
                    spans
                };

                let content_width: usize =
                    content_spans.iter().map(|s| s.text().width()).sum();
                spans.extend(content_spans);

                if content_width < input.min_width() {
                    let padding = input.min_width() - content_width;
                    spans.push(Span::new(" ".repeat(padding)));
                }

                spans.push(Span::new("]"));
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

    pub fn cursor_offset_in_field(&self) -> Option<usize> {
        match self {
            Node::Input(input) if input.is_focused() => {
                let bracket_len = 1;
                let content_offset = input.cursor_offset_in_content();
                Some(bracket_len + content_offset)
            }
            _ => None,
        }
    }
}
