use crate::node::{Display, Node, NodeId, NodeKind, Options, Wrap};
use crate::validators::Validator;
use crossterm::style::Color;

/// Base input node - shared by all input types
pub struct InputNode {
    pub id: NodeId,
    pub label: String,
    pub value: String,
    pub cursor_pos: usize, // Position in characters (not bytes)
    pub validators: Vec<Validator>,
    pub error: Option<String>,
    pub min_width: usize,
}

impl InputNode {
    pub fn new(id: impl Into<NodeId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value: String::new(),
            cursor_pos: 0,
            validators: Vec::new(),
            error: None,
            min_width: 10,
        }
    }

    /// Get cursor position clamped to valid range
    fn clamp_cursor(&self) -> usize {
        self.cursor_pos.min(self.value.chars().count())
    }

    pub fn validate(&self) -> Result<(), String> {
        for validator in &self.validators {
            validator(&self.value)?;
        }
        Ok(())
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text: &str) {
        let mut chars: Vec<char> = self.value.chars().collect();
        let pos = self.clamp_cursor();

        // Insert text at cursor position
        for (i, ch) in text.chars().enumerate() {
            chars.insert(pos + i, ch);
        }

        self.value = chars.into_iter().collect();
        self.cursor_pos = pos + text.chars().count();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_char(&mut self) -> bool {
        if self.cursor_pos == 0 {
            return false;
        }

        let mut chars: Vec<char> = self.value.chars().collect();
        let pos = self.clamp_cursor();

        if pos > 0 {
            chars.remove(pos - 1);
            self.value = chars.into_iter().collect();
            self.cursor_pos = pos - 1;
            true
        } else {
            false
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete_char_forward(&mut self) -> bool {
        let mut chars: Vec<char> = self.value.chars().collect();
        let pos = self.clamp_cursor();

        if pos < chars.len() {
            chars.remove(pos);
            self.value = chars.into_iter().collect();
            true
        } else {
            false
        }
    }

    /// Delete word before cursor (Ctrl+Backspace)
    pub fn delete_word(&mut self) -> bool {
        if self.cursor_pos == 0 {
            return false;
        }

        let mut chars: Vec<char> = self.value.chars().collect();
        let mut pos = self.clamp_cursor();

        // Remove trailing whitespace first
        while pos > 0 && chars.get(pos - 1).is_some_and(|c| c.is_whitespace()) {
            chars.remove(pos - 1);
            pos -= 1;
        }

        // Remove word characters
        while pos > 0 && chars.get(pos - 1).is_some_and(|c| !c.is_whitespace()) {
            chars.remove(pos - 1);
            pos -= 1;
        }

        self.value = chars.into_iter().collect();
        self.cursor_pos = pos;
        true
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) -> bool {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            true
        } else {
            false
        }
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) -> bool {
        let max_pos = self.value.chars().count();
        if self.cursor_pos < max_pos {
            self.cursor_pos += 1;
            true
        } else {
            false
        }
    }

    /// Move cursor to start
    pub fn move_cursor_home(&mut self) -> bool {
        if self.cursor_pos > 0 {
            self.cursor_pos = 0;
            true
        } else {
            false
        }
    }

    /// Move cursor to end
    pub fn move_cursor_end(&mut self) -> bool {
        let max_pos = self.value.chars().count();
        if self.cursor_pos < max_pos {
            self.cursor_pos = max_pos;
            true
        } else {
            false
        }
    }

    /// Clear all text
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor_pos = 0;
    }
}

/// Text input node - single line text input
pub struct TextInputNode {
    pub input: InputNode,
}

/// Builder for TextInput node
pub struct TextInputBuilder {
    input: InputNode,
    opts: Options,
}

impl TextInputBuilder {
    pub fn new(id: NodeId, label: String) -> Self {
        Self {
            input: InputNode::new(id, label),
            opts: Options::default(),
        }
    }

    // Validation methods

    pub fn required(self) -> Self {
        self.validator(crate::validators::required())
    }

    pub fn min_length(self, len: usize) -> Self {
        self.validator(crate::validators::min_length(len))
    }

    pub fn max_length(self, len: usize) -> Self {
        self.validator(crate::validators::max_length(len))
    }

    pub fn min_width(mut self, width: usize) -> Self {
        self.input.min_width = width;
        self
    }

    pub fn validator(mut self, validator: Validator) -> Self {
        self.input.validators.push(validator);
        self
    }

    // Display options

    pub fn with_display(mut self, display: Display) -> Self {
        self.opts.display = display;
        self
    }

    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.opts.wrap = wrap;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.opts.color = Some(color);
        self
    }

    pub fn with_background(mut self, bg: Color) -> Self {
        self.opts.background = Some(bg);
        self
    }

    pub fn bold(mut self) -> Self {
        self.opts.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.opts.italic = true;
        self
    }

    pub fn underline(mut self) -> Self {
        self.opts.underline = true;
        self
    }

    // Build

    pub fn build(self) -> Node {
        Node {
            opts: self.opts,
            kind: NodeKind::TextInput(TextInputNode { input: self.input }),
        }
    }
}

// Auto-build when used in Vec<Node>
impl From<TextInputBuilder> for Node {
    fn from(builder: TextInputBuilder) -> Self {
        builder.build()
    }
}
