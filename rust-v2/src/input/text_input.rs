use crate::input::InputContent;
use crate::input::base::Input;
use crossterm::event::{KeyCode, KeyModifiers};
use unicode_width::UnicodeWidthStr;

/// Zawartość tekstowego inputa
pub struct TextInputContent {
    value: String,
    cursor_pos: usize,
    min_width: usize,
}

impl TextInputContent {
    pub fn new() -> Self {
        Self {
            value: String::new(),
            cursor_pos: 0,
            min_width: 20,
        }
    }

    pub fn with_min_width(mut self, width: usize) -> Self {
        self.min_width = width;
        self
    }
}

impl Default for TextInputContent {
    fn default() -> Self {
        Self::new()
    }
}

impl InputContent for TextInputContent {
    fn render_value(&self) -> String {
        let mut result = self.value.clone();

        // Padding do min_width
        let current_width = result.width();
        let padding_needed = self.min_width.saturating_sub(current_width);
        for _ in 0..padding_needed {
            result.push(' ');
        }

        result
    }

    fn cursor_position(&self) -> usize {
        self.cursor_pos
    }

    fn value(&self) -> &str {
        &self.value
    }

    fn set_value(&mut self, value: String) {
        self.cursor_pos = value.chars().count();
        self.value = value;
    }

    fn handle_char(&mut self, ch: char) {
        let chars: Vec<char> = self.value.chars().collect();
        let mut new_value = String::new();

        for (i, c) in chars.iter().enumerate() {
            if i == self.cursor_pos {
                new_value.push(ch);
            }
            new_value.push(*c);
        }

        if self.cursor_pos >= chars.len() {
            new_value.push(ch);
        }

        self.value = new_value;
        self.cursor_pos += 1;
    }

    fn handle_backspace(&mut self) {
        if self.cursor_pos > 0 {
            let chars: Vec<char> = self.value.chars().collect();
            self.value = chars
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != self.cursor_pos - 1)
                .map(|(_, c)| c)
                .collect();
            self.cursor_pos -= 1;
        }
    }

    fn delete_word(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        let chars: Vec<char> = self.value.chars().collect();

        // Znajdź początek poprzedniego wyrazu
        // 1. Pomiń whitespace za kursorem (w lewo)
        let mut pos = self.cursor_pos;
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // 2. Pomiń non-whitespace (właściwy wyraz)
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }

        // Usuń od `pos` do `cursor_pos`
        let new_value: String = chars
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < pos || *i >= self.cursor_pos)
            .map(|(_, c)| c)
            .collect();

        self.value = new_value;
        self.cursor_pos = pos;
    }

    fn delete_word_forward(&mut self) {
        let chars: Vec<char> = self.value.chars().collect();
        if self.cursor_pos >= chars.len() {
            return;
        }

        // Znajdź koniec następnego wyrazu
        // 1. Pomiń whitespace przed kursorem (w prawo)
        let mut pos = self.cursor_pos;
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }

        // 2. Pomiń non-whitespace (właściwy wyraz)
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }

        // Usuń od `cursor_pos` do `pos`
        let new_value: String = chars
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < self.cursor_pos || *i >= pos)
            .map(|(_, c)| c)
            .collect();

        self.value = new_value;
        // cursor_pos pozostaje bez zmian
    }

    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> bool {
        match code {
            KeyCode::Char(ch) => {
                self.handle_char(ch);
                true
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.value.chars().count() {
                    let chars: Vec<char> = self.value.chars().collect();
                    self.value = chars
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != self.cursor_pos)
                        .map(|(_, c)| c)
                        .collect();
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.value.chars().count() {
                    self.cursor_pos += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.value.chars().count();
                true
            }
            _ => false,
        }
    }
}

/// Tekstowy input - bazowy Input z TextInputContent
pub type TextInput = Input<TextInputContent>;

// Pomocnicze metody dla TextInput
pub fn text_input(label: impl Into<String>) -> TextInput {
    Input::new(label, TextInputContent::new())
}

impl TextInput {
    pub fn with_min_width(mut self, width: usize) -> Self {
        self.content_mut().min_width = width;
        self
    }
}
