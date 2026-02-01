use crate::drawable::{Display, Drawable, Wrap};
use crate::input::{InputContent, InputField, Validator};
use crate::span::Span;
use crate::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::Color;
use std::time::{Duration, Instant};

/// Bazowa struktura dla wszystkich inputów
/// Zarządza: focusem, labelem, walidacją, errorem
pub struct Input<C: InputContent> {
    label: String,
    content: C,
    focused: bool,
    error: Option<String>,
    error_shown_at: Option<Instant>,
    validations: Vec<Validator>,
}

impl<C: InputContent> Input<C> {
    pub fn new(label: impl Into<String>, content: C) -> Self {
        Self {
            label: label.into(),
            content,
            focused: false,
            error: None,
            error_shown_at: None,
            validations: Vec::new(),
        }
    }

    pub fn with_validation(mut self, validator: Validator) -> Self {
        self.validations.push(validator);
        self
    }

    pub fn add_validation(&mut self, validator: Validator) {
        self.validations.push(validator);
    }

    pub fn content(&self) -> &C {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut C {
        &mut self.content
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        if !focused {
            self.clear_error();
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let value = self.content.value();
        for validator in &self.validations {
            validator(value)?;
        }
        Ok(())
    }

    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }

    pub fn show_error(&mut self) {
        if let Err(msg) = self.validate() {
            self.error = Some(msg);
            self.error_shown_at = Some(Instant::now());
        }
    }

    pub fn clear_error(&mut self) {
        self.error = None;
        self.error_shown_at = None;
    }

    /// Sprawdź czy error powinien być ukryty (minęło 3 sekundy)
    /// Zwraca true jeśli error został ukryty (potrzebny re-render)
    pub fn update_error_timer(&mut self) -> bool {
        if let Some(shown_at) = self.error_shown_at {
            if shown_at.elapsed() > Duration::from_secs(3) {
                self.clear_error();
                return true;
            }
        }
        false
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    /// Zwraca pozycję kursora w znakach względem początku contentu
    pub fn content_cursor_position(&self) -> usize {
        self.content.cursor_position()
    }
}

impl<C: InputContent> InputField for Input<C> {
    fn value(&self) -> &str {
        self.content.value()
    }

    fn set_value(&mut self, value: String) {
        self.content.set_value(value);
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.set_focused(focused);
    }

    fn error(&self) -> Option<&str> {
        self.error()
    }

    fn validate(&self) -> Result<(), String> {
        self.validate()
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.content.handle_key(code, modifiers)
    }

    fn handle_char(&mut self, ch: char) {
        self.content.handle_char(ch);
    }

    fn handle_backspace(&mut self) {
        self.content.handle_backspace();
    }

    fn delete_word(&mut self) {
        self.content.delete_word();
    }

    fn delete_word_forward(&mut self) {
        self.content.delete_word_forward();
    }

    fn show_error(&mut self) {
        self.show_error();
    }

    fn clear_error(&mut self) {
        self.clear_error();
    }

    fn update_error_timer(&mut self) -> bool {
        self.update_error_timer()
    }
}

impl<C: InputContent> Drawable for Input<C> {
    fn spans(&self) -> Vec<Span> {
        let mut spans = Vec::new();

        // Label
        spans.push(Span::new(format!("{}: ", self.label)).with_wrap(Wrap::No));

        // Focused: dodaj [ ]
        if self.focused {
            spans.push(Span::new("[").with_wrap(Wrap::No));
        }

        // Jeśli error jest wyświetlany, pokaż error ZAMIAST zawartości
        if let Some(error) = &self.error {
            spans.push(
                Span::new(format!("✗ {}", error))
                    .with_style(Style::default().with_fg(Color::Red))
                    .with_wrap(Wrap::No),
            );
        } else {
            // Zawartość inputa
            let is_valid = self.is_valid();

            // Sprawdź czy content ma custom rendering (np. IpInput z boldem)
            if let Some(mut content_spans) = self.content.render_spans() {
                // Jeśli nieważne, dodaj czerwony kolor do wszystkich spanów
                if !is_valid && self.focused {
                    for span in &mut content_spans {
                        let mut style = span.style().clone();
                        style = style.with_fg(Color::Red);
                        *span = span.clone().with_style(style);
                    }
                }
                spans.extend(content_spans);
            } else {
                // Domyślny rendering jako string
                let style = if is_valid || !self.focused {
                    Style::default()
                } else {
                    Style::default().with_fg(Color::Red)
                };

                let content_str = self.content.render_value();
                spans.push(Span::new(content_str).with_style(style).with_wrap(Wrap::No));
            }
        }

        // Zamknij [ ]
        if self.focused {
            spans.push(Span::new("]").with_wrap(Wrap::No));
        }

        spans
    }

    fn display(&self) -> Display {
        Display::Block
    }

    fn cursor_offset(&self) -> Option<usize> {
        if self.focused && self.error.is_none() {
            // Offset = label + ": " + "["
            let label_width = self.label.chars().count() + 2; // ": "
            let bracket_width = 1; // "["
            let content_offset = self.content.cursor_position();
            Some(label_width + bracket_width + content_offset)
        } else {
            None
        }
    }
}
