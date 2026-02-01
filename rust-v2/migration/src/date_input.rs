use crate::input::{Input, InputBase, KeyResult, NodeId, Validator};
use crate::span::Span;
use crate::style::Style;
use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::Color;
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

impl SegmentType {
    fn min_value(&self) -> u32 {
        match self {
            SegmentType::Year => 1900,
            SegmentType::Month | SegmentType::Day => 1,
            _ => 0,
        }
    }

    fn max_value(&self) -> u32 {
        match self {
            SegmentType::Year => 2100,
            SegmentType::Month => 12,
            SegmentType::Day => 31,
            SegmentType::Hour => 23,
            SegmentType::Minute | SegmentType::Second => 59,
        }
    }

    fn length(&self) -> usize {
        match self {
            SegmentType::Year => 4,
            _ => 2,
        }
    }

    fn from_token(token: &str) -> Option<Self> {
        match token {
            "YYYY" => Some(SegmentType::Year),
            "MM" => Some(SegmentType::Month),
            "DD" => Some(SegmentType::Day),
            "HH" => Some(SegmentType::Hour),
            "mm" => Some(SegmentType::Minute),
            "ss" => Some(SegmentType::Second),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct DateSegment {
    segment_type: SegmentType,
    value: String,
}

impl DateSegment {
    fn new(segment_type: SegmentType) -> Self {
        Self {
            segment_type,
            value: String::new(),
        }
    }

    fn numeric_value(&self) -> u32 {
        self.value.parse().unwrap_or(0)
    }

    fn increment(&mut self) {
        let current = self.numeric_value();
        let max = self.segment_type.max_value();
        let min = self.segment_type.min_value();
        let next = if current >= max || current < min {
            min
        } else {
            current + 1
        };
        self.value = format!("{:0width$}", next, width = self.segment_type.length());
    }

    fn decrement(&mut self) {
        let current = self.numeric_value();
        let max = self.segment_type.max_value();
        let min = self.segment_type.min_value();
        let prev = if current <= min || current == 0 {
            max
        } else {
            current - 1
        };
        self.value = format!("{:0width$}", prev, width = self.segment_type.length());
    }

    fn insert_digit(&mut self, digit: char) -> bool {
        if !digit.is_ascii_digit() {
            return false;
        }
        let max_len = self.segment_type.length();
        if self.value.len() >= max_len {
            self.value = digit.to_string();
            return true;
        }
        self.value.push(digit);
        if let Ok(val) = self.value.parse::<u32>() {
            if val > self.segment_type.max_value() {
                self.value = digit.to_string();
            }
        }
        true
    }

    fn delete_digit(&mut self) -> bool {
        if self.value.is_empty() {
            return false;
        }
        self.value.pop();
        true
    }

    fn display_string(&self) -> String {
        let len = self.segment_type.length();
        if self.value.is_empty() {
            "_".repeat(len)
        } else if self.value.len() < len {
            format!("{}{}", self.value, "_".repeat(len - self.value.len()))
        } else {
            self.value.clone()
        }
    }
}

pub struct DateInput {
    base: InputBase,
    format: String,
    segments: Vec<DateSegment>,
    separators: Vec<String>,
    focused_segment: usize,
}

impl DateInput {
    pub fn new(id: impl Into<String>, label: impl Into<String>, format: impl Into<String>) -> Self {
        let format_str = format.into();
        let (segments, separators) = Self::parse_format(&format_str);

        Self {
            base: InputBase::new(id, label),
            format: format_str,
            segments,
            separators,
            focused_segment: 0,
        }
    }

    pub fn with_min_width(mut self, width: usize) -> Self {
        self.base = self.base.with_min_width(width);
        self
    }

    pub fn with_validator(mut self, validator: Validator) -> Self {
        self.base = self.base.with_validator(validator);
        self
    }

    fn parse_format(format: &str) -> (Vec<DateSegment>, Vec<String>) {
        let mut segments = Vec::new();
        let mut separators = Vec::new();
        let mut current_sep = String::new();
        let mut chars = format.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch.is_alphabetic() {
                let mut token = String::from(ch);
                while let Some(&next_ch) = chars.peek() {
                    if next_ch == ch {
                        token.push(next_ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Some(seg_type) = SegmentType::from_token(&token) {
                    separators.push(current_sep.clone());
                    current_sep.clear();
                    segments.push(DateSegment::new(seg_type));
                } else {
                    current_sep.push_str(&token);
                }
            } else {
                current_sep.push(ch);
            }
        }
        separators.push(current_sep);
        (segments, separators)
    }

    pub fn display_string(&self) -> String {
        let mut result = String::new();
        for (i, segment) in self.segments.iter().enumerate() {
            if i < self.separators.len() {
                result.push_str(&self.separators[i]);
            }
            result.push_str(&segment.display_string());
        }
        if self.segments.len() < self.separators.len() {
            result.push_str(&self.separators[self.segments.len()]);
        }
        result
    }

    fn move_next(&mut self) -> bool {
        if let Some(segment) = self.segments.get_mut(self.focused_segment) {
            if !segment.value.is_empty() {
                let len = segment.segment_type.length();
                if segment.value.len() < len {
                    segment.value = format!("{:0width$}", segment.value, width = len);
                }
            }
        }

        if self.focused_segment + 1 < self.segments.len() {
            self.focused_segment += 1;
            true
        } else {
            false
        }
    }

    fn move_prev(&mut self) -> bool {
        if self.focused_segment > 0 {
            self.focused_segment -= 1;
            true
        } else {
            false
        }
    }
}

impl Input for DateInput {
    fn id(&self) -> &NodeId {
        &self.base.id
    }

    fn label(&self) -> &str {
        &self.base.label
    }

    fn value(&self) -> String {
        self.display_string()
    }

    fn set_value(&mut self, _value: String) {}

    fn is_focused(&self) -> bool {
        self.base.focused
    }

    fn set_focused(&mut self, focused: bool) {
        self.base.focused = focused;
        if !focused {
            self.base.error = None;
        }
    }

    fn error(&self) -> Option<&str> {
        self.base.error.as_deref()
    }

    fn set_error(&mut self, error: Option<String>) {
        self.base.error = error;
    }

    fn cursor_pos(&self) -> usize {
        self.focused_segment
    }

    fn min_width(&self) -> usize {
        self.base.min_width
    }

    fn validators(&self) -> &[Validator] {
        &self.base.validators
    }

    fn handle_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> KeyResult {
        match code {
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                if let Some(segment) = self.segments.get_mut(self.focused_segment) {
                    segment.insert_digit(ch);
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Backspace => {
                if let Some(segment) = self.segments.get_mut(self.focused_segment) {
                    segment.delete_digit();
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Left => {
                if self.move_prev() {
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Right | KeyCode::Char('/') | KeyCode::Char(':') => {
                if self.move_next() {
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Up => {
                if let Some(segment) = self.segments.get_mut(self.focused_segment) {
                    segment.increment();
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Down => {
                if let Some(segment) = self.segments.get_mut(self.focused_segment) {
                    segment.decrement();
                    KeyResult::Handled
                } else {
                    KeyResult::NotHandled
                }
            }
            KeyCode::Enter => {
                if let Some(segment) = self.segments.get_mut(self.focused_segment) {
                    if !segment.value.is_empty() {
                        let len = segment.segment_type.length();
                        if segment.value.len() < len {
                            segment.value = format!("{:0width$}", segment.value, width = len);
                        }
                    }
                }
                KeyResult::Submit
            }
            _ => KeyResult::NotHandled,
        }
    }

    fn render_content(&self) -> Vec<Span> {
        let mut spans = Vec::new();

        for (i, segment) in self.segments.iter().enumerate() {
            if i < self.separators.len() && !self.separators[i].is_empty() {
                spans.push(Span::new(&self.separators[i]));
            }

            let style = if i == self.focused_segment && self.base.focused {
                Style::new().with_fg(Color::Yellow)
            } else {
                Style::default()
            };
            spans.push(Span::new(segment.display_string()).with_style(style));
        }

        if self.segments.len() < self.separators.len() {
            let last_sep = &self.separators[self.segments.len()];
            if !last_sep.is_empty() {
                spans.push(Span::new(last_sep));
            }
        }

        let content_width = self.display_string().width();
        if content_width < self.base.min_width {
            let padding = self.base.min_width - content_width;
            spans.push(Span::new(" ".repeat(padding)));
        }

        spans
    }

    fn cursor_offset_in_content(&self) -> usize {
        let mut offset = 0;
        for i in 0..self.focused_segment {
            if i < self.separators.len() {
                offset += self.separators[i].width();
            }
            offset += self.segments[i].display_string().width();
        }
        if self.focused_segment < self.separators.len() {
            offset += self.separators[self.focused_segment].width();
        }
        offset
    }
}
