use crate::drawable::Wrap;
use crate::style::Style;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    text: String,
    style: Style,
    wrap: Wrap,
}

impl Span {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default(),
            wrap: Wrap::Yes,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
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

    pub fn width(&self) -> usize {
        self.text.chars().count()
    }

    pub fn split_at_width(&self, max: usize) -> (Span, Option<Span>) {
        if max == 0 {
            return (Span::new(String::new()), Some(self.clone()));
        }

        if self.width() <= max {
            return (self.clone(), None);
        }

        let mut left = String::new();
        let mut iter = self.text.chars();
        for _ in 0..max {
            if let Some(ch) = iter.next() {
                left.push(ch);
            } else {
                return (self.clone(), None);
            }
        }
        let right: String = iter.collect();
        let tail = if right.is_empty() {
            None
        } else {
            Some(
                Span::new(right)
                    .with_style(self.style.clone())
                    .with_wrap(self.wrap),
            )
        };
        (
            Span::new(left)
                .with_style(self.style.clone())
                .with_wrap(self.wrap),
            tail,
        )
    }
}
