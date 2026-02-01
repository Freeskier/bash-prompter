# Migration - Nowa Architektura

## Podsumowanie

Projekt `migration/` to kompletnie przepisana aplikacja bash-prompter z czystą, trait-based architekturą.

## Kluczowe Zmiany

### 1. Trait-Based Design

**Przed:**
```rust
enum NodeKind {
    Text(TextNode),
    TextInput(TextInputNode),
    DateInput(DateInputNode),
}
```

**Po:**
```rust
pub trait Input: Send {
    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> KeyResult;
    fn render(&self) -> Vec<Span>;
    fn validate(&self) -> Result<(), String>;
    // ... inne metody
}

enum Node {
    Text(String),
    Input(Box<dyn Input>),
}
```

### 2. Uproszczony Step

**Przed (25 metod):**
```rust
trait StepExt {
    fn date_insert_digit(&mut self, id: &NodeId, digit: char) -> bool;
    fn date_move_next(&mut self, id: &NodeId) -> bool;
    fn move_cursor_left(&mut self, id: &NodeId) -> bool;
    fn delete_char(&mut self, id: &NodeId) -> bool;
    // ... +20 więcej metod
}
```

**Po (4 metody):**
```rust
trait StepExt {
    fn find_input(&self, id: &str) -> Option<&dyn Input>;
    fn find_input_mut(&mut self, id: &str) -> Option<&mut dyn Input>;
    fn validate_all(&self) -> Vec<(NodeId, String)>;
    fn values(&self) -> Vec<(NodeId, String)>;
}
```

### 3. Uproszczony Event Loop

**Przed (~60 linii):**
```rust
match key_event.code {
    KeyCode::Char(ch) => {
        if is_date_input {
            if ch.is_ascii_digit() {
                step.date_insert_digit(focused_id, ch);
            }
        } else {
            step.insert_text(focused_id, &ch.to_string());
        }
    }
    KeyCode::Backspace => {
        if is_date_input {
            step.date_delete_digit(focused_id);
        } else {
            step.delete_char(focused_id);
        }
    }
    // ... +40 linii
}
```

**Po (~5 linii):**
```rust
if let Some(Node::Input(input)) = step.get_mut(current_idx) {
    let result = input.handle_key(key_event.code, key_event.modifiers);
    if result == KeyResult::Submit {
        // walidacja i submit
    }
}
```

### 4. InputBase - DRY dla wszystkich inputów

```rust
pub struct InputBase {
    pub id: NodeId,
    pub label: String,
    pub focused: bool,
    pub error: Option<String>,
    pub validators: Vec<Validator>,
    pub min_width: usize,
}
```

Każdy input (TextInput, DateInput) zawiera `InputBase` i używa composition pattern.

### 5. Każdy Input Obsługuje Swoje Eventy

**TextInput:**
```rust
impl Input for TextInput {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> KeyResult {
        match code {
            KeyCode::Char(ch) => { self.handle_char(ch); KeyResult::Handled }
            KeyCode::Backspace => { self.handle_backspace(); KeyResult::Handled }
            KeyCode::Left => { self.move_left(); KeyResult::Handled }
            KeyCode::Right => { self.move_right(); KeyResult::Handled }
            KeyCode::Enter => KeyResult::Submit,
            _ => KeyResult::NotHandled,
        }
    }
}
```

**DateInput:**
```rust
impl Input for DateInput {
    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> KeyResult {
        match code {
            KeyCode::Char(ch) if ch.is_ascii_digit() => { /* insert digit */ }
            KeyCode::Up => { /* increment segment */ }
            KeyCode::Down => { /* decrement segment */ }
            KeyCode::Left => { /* prev segment */ }
            KeyCode::Right => { /* next segment */ }
            // ... date-specific logic
        }
    }
}
```

## Struktura Plików

```
migration/src/
├── main.rs              # Prosty event loop (~150 linii)
├── lib.rs               # Module exports
├── input.rs             # Trait Input + InputBase
├── text_input.rs        # impl Input for TextInput
├── date_input.rs        # impl Input for DateInput
├── node.rs              # Node enum + Step type
├── renderer.rs          # Prosty renderer (~40 linii)
├── span.rs              # [copied] Span rendering
├── style.rs             # [copied] Style system
├── frame.rs             # [copied] Frame building
├── validators.rs        # [copied] Validators
└── terminal_state.rs    # [copied] Terminal utils
```

## Dodanie Nowego Typu Inputa

### Przykład: IpInput

**1 plik, 0 zmian w istniejącym kodzie:**

```rust
// src/ip_input.rs
use crate::input::{Input, InputBase, KeyResult, NodeId, Validator};
use crate::span::Span;
use crossterm::event::{KeyCode, KeyModifiers};

pub struct IpInput {
    base: InputBase,
    segments: [u8; 4],
    focused_segment: usize,
}

impl IpInput {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            base: InputBase::new(id, label),
            segments: [0, 0, 0, 0],
            focused_segment: 0,
        }
    }
}

impl Input for IpInput {
    fn id(&self) -> &NodeId { &self.base.id }
    fn label(&self) -> &str { &self.base.label }
    fn value(&self) -> String {
        format!("{}.{}.{}.{}", 
                self.segments[0], self.segments[1], 
                self.segments[2], self.segments[3])
    }
    
    fn handle_key(&mut self, code: KeyCode, _mods: KeyModifiers) -> KeyResult {
        match code {
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                // Insert digit into current segment
                KeyResult::Handled
            }
            KeyCode::Left => {
                if self.focused_segment > 0 {
                    self.focused_segment -= 1;
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Right | KeyCode::Char('.') => {
                if self.focused_segment < 3 {
                    self.focused_segment += 1;
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Enter => KeyResult::Submit,
            _ => KeyResult::NotHandled,
        }
    }
    
    fn render(&self) -> Vec<Span> {
        // Render IP address with highlight on focused segment
        vec![]
    }
    
    // ... pozostałe metody
}
```

**Użycie:**
```rust
let step: Step = vec![
    Node::text("Server Configuration"),
    Node::input(IpInput::new("server_ip", "Server IP")),  // DZIAŁA!
];
```

## Metryki

| Metryka | Stary Projekt | Migration | Zmiana |
|---------|---------------|-----------|--------|
| Metody w StepExt | 25 | 4 | -84% |
| Event loop (linii) | ~60 | ~5 | -92% |
| Dodanie nowego inputa | ~800 linii w 5 plikach | ~150 linii w 1 pliku | -81% |
| main.rs (linii) | 250 | 150 | -40% |

## Korzyści

1. **Extensibility**: Nowy typ inputa = 1 plik, implementacja traitu
2. **Type Safety**: Rust trait system wymusza kompletność
3. **Separation of Concerns**: Każdy input zarządza swoją logiką
4. **DRY**: Wspólna funkcjonalność w InputBase
5. **Clean Code**: Brak komentarzy, klarowna struktura
6. **Maintainability**: Zmiana w jednym inputcie nie wpływa na inne

## Następne Kroki

1. Dodać więcej inputów: IpInput, SelectInput, CheckboxInput
2. Ewentualnie dodać EventEmitter dla reaktywności
3. Layout engine dla lepszego pozycjonowania
4. Theming system
