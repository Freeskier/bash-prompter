use crate::drawable::{Display, Drawable};
use crate::input::InputField;
use crate::input::ip_input::IpInput;
use crate::input::text_input::TextInput;
use crate::span::Span;
use crate::text::Text;

/// Wrapper pozwalający mieszać różne typy Drawable w jednej kolekcji
pub enum FormItem {
    TextInput(TextInput),
    IpInput(IpInput),
    Text(Text),
}

impl FormItem {
    /// Zwraca referencję do InputField jeśli to jest input (wspólny interfejs!)
    pub fn as_input(&self) -> Option<&dyn InputField> {
        match self {
            FormItem::TextInput(input) => Some(input as &dyn InputField),
            FormItem::IpInput(input) => Some(input as &dyn InputField),
            FormItem::Text(_) => None,
        }
    }

    /// Zwraca mutable referencję do InputField jeśli to jest input
    pub fn as_input_mut(&mut self) -> Option<&mut dyn InputField> {
        match self {
            FormItem::TextInput(input) => Some(input as &mut dyn InputField),
            FormItem::IpInput(input) => Some(input as &mut dyn InputField),
            FormItem::Text(_) => None,
        }
    }

    /// Zwraca referencję do TextInput jeśli to jest TextInput (dla specyficznych operacji)
    pub fn as_text_input(&self) -> Option<&TextInput> {
        match self {
            FormItem::TextInput(input) => Some(input),
            _ => None,
        }
    }

    /// Zwraca mutable referencję do TextInput jeśli to jest TextInput
    pub fn as_text_input_mut(&mut self) -> Option<&mut TextInput> {
        match self {
            FormItem::TextInput(input) => Some(input),
            _ => None,
        }
    }

    /// Zwraca referencję do IpInput jeśli to jest IpInput (dla specyficznych operacji)
    pub fn as_ip_input(&self) -> Option<&IpInput> {
        match self {
            FormItem::IpInput(input) => Some(input),
            _ => None,
        }
    }

    /// Zwraca mutable referencję do IpInput jeśli to jest IpInput
    pub fn as_ip_input_mut(&mut self) -> Option<&mut IpInput> {
        match self {
            FormItem::IpInput(input) => Some(input),
            _ => None,
        }
    }

    /// Sprawdza czy to jest input (a nie text)
    pub fn is_input(&self) -> bool {
        matches!(self, FormItem::TextInput(_) | FormItem::IpInput(_))
    }
}

impl Drawable for FormItem {
    fn spans(&self) -> Vec<Span> {
        match self {
            FormItem::TextInput(input) => input.spans(),
            FormItem::IpInput(input) => input.spans(),
            FormItem::Text(text) => text.spans(),
        }
    }

    fn display(&self) -> Display {
        match self {
            FormItem::TextInput(input) => input.display(),
            FormItem::IpInput(input) => input.display(),
            FormItem::Text(text) => text.display(),
        }
    }

    fn cursor_offset(&self) -> Option<usize> {
        match self {
            FormItem::TextInput(input) => input.cursor_offset(),
            FormItem::IpInput(input) => input.cursor_offset(),
            FormItem::Text(_) => None,
        }
    }
}

// Convenience conversion traits
impl From<TextInput> for FormItem {
    fn from(input: TextInput) -> Self {
        FormItem::TextInput(input)
    }
}

impl From<IpInput> for FormItem {
    fn from(input: IpInput) -> Self {
        FormItem::IpInput(input)
    }
}

impl From<Text> for FormItem {
    fn from(text: Text) -> Self {
        FormItem::Text(text)
    }
}
