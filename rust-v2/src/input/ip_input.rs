use crate::input::InputContent;
use crate::input::base::Input;
use crate::span::Span;
use crate::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::Attribute;

/// Zawartość IP address inputa (IPv4)
pub struct IpInputContent {
    octets: [u8; 4],        // Wartości oktetów (0-255)
    current_octet: usize,   // Który oktet edytujemy (0-3)
    editing_buffer: String, // Buffer dla aktualnie edytowanego oktetu
}

impl IpInputContent {
    pub fn new() -> Self {
        Self {
            octets: [0, 0, 0, 0],
            current_octet: 0,
            editing_buffer: String::from("0"),
        }
    }

    pub fn with_value(ip: [u8; 4]) -> Self {
        Self {
            octets: ip,
            current_octet: 0,
            editing_buffer: ip[0].to_string(),
        }
    }

    /// Zwiększ wartość aktualnego oktetu
    fn increment_octet(&mut self) {
        let current_value = self.octets[self.current_octet];
        self.octets[self.current_octet] = current_value.saturating_add(1);
        self.editing_buffer = self.octets[self.current_octet].to_string();
    }

    /// Zmniejsz wartość aktualnego oktetu
    fn decrement_octet(&mut self) {
        let current_value = self.octets[self.current_octet];
        self.octets[self.current_octet] = current_value.saturating_sub(1);
        self.editing_buffer = self.octets[self.current_octet].to_string();
    }

    /// Przejdź do następnego oktetu
    fn next_octet(&mut self) {
        self.commit_buffer();
        if self.current_octet < 3 {
            self.current_octet += 1;
            self.editing_buffer = self.octets[self.current_octet].to_string();
        }
    }

    /// Przejdź do poprzedniego oktetu
    fn prev_octet(&mut self) {
        self.commit_buffer();
        if self.current_octet > 0 {
            self.current_octet -= 1;
            self.editing_buffer = self.octets[self.current_octet].to_string();
        }
    }

    /// Zapisz buffer do aktualnego oktetu
    fn commit_buffer(&mut self) {
        if let Ok(val) = self.editing_buffer.parse::<u16>() {
            self.octets[self.current_octet] = val.min(255) as u8;
        }
        // Jeśli parsing się nie powiódł, zostaw poprzednią wartość
    }

    /// Dodaj cyfrę do bufferu
    fn add_digit(&mut self, digit: char) {
        // Jeśli buffer to "0", zastąp go
        if self.editing_buffer == "0" {
            self.editing_buffer.clear();
        }

        let test_buffer = format!("{}{}", self.editing_buffer, digit);
        if let Ok(val) = test_buffer.parse::<u16>() {
            if val <= 255 {
                self.editing_buffer = test_buffer;

                // Auto-przejście po 3 cyfrach
                if self.editing_buffer.len() == 3 && self.current_octet < 3 {
                    self.next_octet();
                }
            }
            // Jeśli przekracza 255, ignoruj cyfrę
        }
    }

    /// Usuń ostatnią cyfrę
    fn remove_digit(&mut self) {
        self.editing_buffer.pop();
        if self.editing_buffer.is_empty() {
            self.editing_buffer = String::from("0");
        }
    }

    /// Pobierz IP jako string (np. "192.168.1.1")
    pub fn to_ip_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }

    /// Pobierz IP jako array
    pub fn to_array(&self) -> [u8; 4] {
        self.octets
    }
}

impl Default for IpInputContent {
    fn default() -> Self {
        Self::new()
    }
}

impl InputContent for IpInputContent {
    fn render_value(&self) -> String {
        // Fallback - używane dla pozycji kursora
        self.to_ip_string()
    }

    fn render_spans(&self) -> Option<Vec<Span>> {
        let mut spans = Vec::new();

        for i in 0..4 {
            // Separator kropka (oprócz pierwszego)
            if i > 0 {
                spans.push(Span::new("."));
            }

            // Wartość oktetu
            let octet_str = if i == self.current_octet {
                // Aktualnie edytowany oktet - użyj bufferu
                self.editing_buffer.clone()
            } else {
                self.octets[i].to_string()
            };

            // Styl: bold dla aktualnego oktetu
            if i == self.current_octet {
                spans.push(
                    Span::new(octet_str)
                        .with_style(Style::default().with_attribute(Attribute::Bold)),
                );
            } else {
                spans.push(Span::new(octet_str));
            }
        }

        Some(spans)
    }

    fn cursor_position(&self) -> usize {
        // Oblicz pozycję kursora
        let mut pos = 0;

        // Dodaj długości poprzednich oktetów + kropki
        for i in 0..self.current_octet {
            pos += self.octets[i].to_string().len() + 1; // +1 for '.'
        }

        // Dodaj pozycję w bufferze (kursor na końcu)
        pos += self.editing_buffer.len();

        pos
    }

    fn value(&self) -> &str {
        // Dla walidacji zwracamy string - używamy static lifetime trick
        // W prawdziwej implementacji możesz użyć Cow<'static, str> albo cache
        // Na razie zwracamy empty string - walidacja sprawdzi oktety bezpośrednio
        ""
    }

    fn set_value(&mut self, value: String) {
        // Parse IP address string (np. "192.168.1.1")
        let parts: Vec<&str> = value.split('.').collect();
        if parts.len() == 4 {
            for (i, part) in parts.iter().enumerate() {
                if let Ok(val) = part.parse::<u8>() {
                    self.octets[i] = val;
                }
            }
        }
        self.current_octet = 0;
        self.editing_buffer = self.octets[0].to_string();
    }

    fn handle_char(&mut self, ch: char) {
        if ch.is_ascii_digit() {
            self.add_digit(ch);
        } else if ch == '.' {
            self.next_octet();
        }
    }

    fn handle_backspace(&mut self) {
        if self.editing_buffer.len() > 1 || self.editing_buffer != "0" {
            self.remove_digit();
        } else if self.current_octet > 0 {
            // Jeśli buffer jest pusty i nie jesteśmy na pierwszym oktecie,
            // przejdź do poprzedniego
            self.prev_octet();
        }
    }

    fn delete_word(&mut self) {
        // W IP input usuwamy cały oktet (resetujemy do 0)
        self.octets[self.current_octet] = 0;
        self.editing_buffer = String::from("0");
    }

    fn delete_word_forward(&mut self) {
        // Możesz zaimplementować inaczej, np. wyczyść wszystkie następne oktety
        if self.current_octet < 3 {
            for i in (self.current_octet + 1)..4 {
                self.octets[i] = 0;
            }
        }
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        match code {
            KeyCode::Char(ch) => {
                // Ignoruj Ctrl kombinacje (obsługiwane przez InputManager)
                if modifiers.contains(KeyModifiers::CONTROL) {
                    return false;
                }
                self.handle_char(ch);
                true
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                true
            }
            KeyCode::Delete => {
                // Delete wyczyść aktualny oktet
                self.editing_buffer = String::from("0");
                self.octets[self.current_octet] = 0;
                true
            }
            KeyCode::Left => {
                self.prev_octet();
                true
            }
            KeyCode::Right => {
                self.next_octet();
                true
            }
            KeyCode::Up => {
                self.increment_octet();
                true
            }
            KeyCode::Down => {
                self.decrement_octet();
                true
            }
            KeyCode::Home => {
                self.commit_buffer();
                self.current_octet = 0;
                self.editing_buffer = self.octets[0].to_string();
                true
            }
            KeyCode::End => {
                self.commit_buffer();
                self.current_octet = 3;
                self.editing_buffer = self.octets[3].to_string();
                true
            }
            _ => false,
        }
    }
}

/// IP address input - bazowy Input z IpInputContent
pub type IpInput = Input<IpInputContent>;

// Pomocnicza funkcja do tworzenia IP inputa
pub fn ip_input(label: impl Into<String>) -> IpInput {
    Input::new(label, IpInputContent::new())
}

impl IpInput {
    /// Utwórz z domyślną wartością
    pub fn with_default(mut self, ip: [u8; 4]) -> Self {
        self.content_mut().octets = ip;
        self.content_mut().editing_buffer = ip[0].to_string();
        self
    }

    /// Pobierz wartość IP jako array
    pub fn ip_value(&self) -> [u8; 4] {
        self.content().to_array()
    }

    /// Pobierz wartość IP jako string
    pub fn ip_string(&self) -> String {
        self.content().to_ip_string()
    }
}

// Validator dla IP address
pub fn valid_ip() -> crate::input::Validator {
    Box::new(|_value: &str| {
        // IP input zawsze ma valid oktety (0-255), więc zawsze OK
        // Możesz dodać dodatkowe warunki, np. nie może być 0.0.0.0
        Ok(())
    })
}

pub fn not_localhost() -> crate::input::Validator {
    Box::new(|_value: &str| {
        // Ta walidacja wymaga dostępu do oktetów
        // W prawdziwej implementacji przekazałbyś &IpInputContent
        // Na razie placeholder
        Ok(())
    })
}
