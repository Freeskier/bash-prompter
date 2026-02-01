pub mod base;
pub mod ip_input;
pub mod text_input;
pub mod validators;

use crate::span::Span;
use crossterm::event::{KeyCode, KeyModifiers};

/// Validator zwraca Ok(()) jeśli walidacja przeszła, Err(message) w przeciwnym razie
pub type Validator = Box<dyn Fn(&str) -> Result<(), String>>;

/// Trait dla specyficznej logiki każdego typu inputa
/// Każdy typ inputa (TextInput, IpAddressInput, etc.) implementuje ten trait
pub trait InputContent {
    /// Zwraca wyrenderowaną wartość (bez kursora - kursor będzie renderowany przez terminal)
    fn render_value(&self) -> String;

    /// Zwraca spany dla customowego renderowania (opcjonalne - domyślnie używa render_value)
    fn render_spans(&self) -> Option<Vec<Span>> {
        None
    }

    /// Pobierz pozycję kursora w znakach (0-based, używane do ustawienia prawdziwego kursora)
    fn cursor_position(&self) -> usize;

    /// Pobierz surową wartość (bez formatowania)
    fn value(&self) -> &str;

    /// Ustaw wartość
    fn set_value(&mut self, value: String);

    /// Obsłuż wpisanie znaku
    fn handle_char(&mut self, ch: char);

    /// Obsłuż backspace
    fn handle_backspace(&mut self);

    /// Usuń poprzedni wyraz (Ctrl+Backspace)
    fn delete_word(&mut self);

    /// Usuń następny wyraz (Ctrl+Delete)
    fn delete_word_forward(&mut self);

    /// Obsłuż naciśnięcie klawisza
    /// Zwraca true jeśli event został obsłużony
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool;
}

/// Trait dla wszystkich pól inputowych (bazowa funkcjonalność)
pub trait InputField {
    /// Pobierz aktualną wartość
    fn value(&self) -> &str;

    /// Ustaw wartość
    fn set_value(&mut self, value: String);

    /// Czy input jest aktywny (focused)
    fn is_focused(&self) -> bool;

    /// Ustaw stan focus
    fn set_focused(&mut self, focused: bool);

    /// Pobierz błąd walidacji (jeśli jest wyświetlany)
    fn error(&self) -> Option<&str>;

    /// Wykonaj walidację - zwraca pierwszy błąd lub Ok
    fn validate(&self) -> Result<(), String>;

    /// Obsłuż naciśnięcie klawisza
    /// Zwraca true jeśli event został obsłużony
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool;

    /// Dodaj znak do wartości
    fn handle_char(&mut self, ch: char);

    /// Usuń znak (backspace)
    fn handle_backspace(&mut self);

    /// Usuń poprzedni wyraz (Ctrl+Backspace)
    fn delete_word(&mut self);

    /// Usuń następny wyraz (Ctrl+Delete)
    fn delete_word_forward(&mut self);

    /// Pokaż błąd walidacji (np. po Submit)
    fn show_error(&mut self);

    /// Usuń wyświetlany błąd
    fn clear_error(&mut self);

    /// Zaktualizuj timer errora (wywołaj w każdej iteracji)
    /// Zwraca true jeśli error został ukryty (potrzebny re-render)
    fn update_error_timer(&mut self) -> bool;
}
