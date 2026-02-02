use crate::style::{Color, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub prompt: Style,
    pub hint: Style,
    pub error: Style,
    pub placeholder: Style,
    pub focused: Style,
}

impl Theme {
    pub fn default_theme() -> Self {
        Self {
            prompt: Style::new().with_bold(),
            hint: Style::new().with_color(Color::DarkGrey),
            error: Style::new()
                .with_color(Color::Red)
                .with_bold(),
            placeholder: Style::new().with_color(Color::DarkGrey),
            focused: Style::new().with_bold(),
        }
    }
}

pub fn prompt_style() -> Style {
    Theme::default_theme().prompt
}

pub fn hint_style() -> Style {
    Theme::default_theme().hint
}

pub fn error_style() -> Style {
    Theme::default_theme().error
}

pub fn placeholder_style() -> Style {
    Theme::default_theme().placeholder
}

pub fn focused_style() -> Style {
    Theme::default_theme().focused
}
