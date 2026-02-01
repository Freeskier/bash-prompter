use crate::input::InputNode;
use crate::style::Style;
use crossterm::style::Color;

// Re-export for convenience
pub use crate::input::{TextInputBuilder, TextInputNode};

/// Display mode for nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
}

/// Text wrapping behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    Yes,
    No,
}

/// Unique identifier for nodes (especially inputs)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Display and styling options for a node
#[derive(Debug, Clone)]
pub struct Options {
    pub display: Display,
    pub wrap: Wrap,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            display: Display::Block,
            wrap: Wrap::Yes,
            color: None,
            background: None,
            bold: false,
            italic: false,
            underline: false,
        }
    }
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    /// Convert Options to Style
    pub fn to_style(&self) -> Style {
        let mut style = Style::new();

        if let Some(fg) = self.color {
            style = style.with_fg(fg);
        }

        if let Some(bg) = self.background {
            style = style.with_bg(bg);
        }

        if self.bold {
            style = style.with_attribute(crossterm::style::Attribute::Bold);
        }

        if self.italic {
            style = style.with_attribute(crossterm::style::Attribute::Italic);
        }

        if self.underline {
            style = style.with_attribute(crossterm::style::Attribute::Underlined);
        }

        style
    }
}

/// Text node - just displays static text
#[derive(Debug, Clone)]
pub struct TextNode {
    pub text: String,
}

/// The kind of node - what it represents semantically
pub enum NodeKind {
    Text(TextNode),
    TextInput(TextInputNode),
}

impl NodeKind {
    /// Get the input ID if this is an input node
    pub fn input_id(&self) -> Option<&NodeId> {
        match self {
            NodeKind::Text(_) => None,
            NodeKind::TextInput(text_input) => Some(&text_input.input.id),
        }
    }

    /// Get mutable reference to value if this is an input
    pub fn value_mut(&mut self) -> Option<&mut String> {
        match self {
            NodeKind::Text(_) => None,
            NodeKind::TextInput(text_input) => Some(&mut text_input.input.value),
        }
    }

    /// Get reference to value if this is an input
    pub fn value(&self) -> Option<&str> {
        match self {
            NodeKind::Text(_) => None,
            NodeKind::TextInput(text_input) => Some(&text_input.input.value),
        }
    }

    /// Get reference to InputNode if this is an input
    pub fn input(&self) -> Option<&InputNode> {
        match self {
            NodeKind::Text(_) => None,
            NodeKind::TextInput(text_input) => Some(&text_input.input),
        }
    }

    /// Get mutable reference to InputNode if this is an input
    pub fn input_mut(&mut self) -> Option<&mut InputNode> {
        match self {
            NodeKind::Text(_) => None,
            NodeKind::TextInput(text_input) => Some(&mut text_input.input),
        }
    }
}

/// A node in the step - combines kind (what) with options (how to display)
pub struct Node {
    pub opts: Options,
    pub kind: NodeKind,
}

impl Node {
    /// Create a text node
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            opts: Options::default(),
            kind: NodeKind::Text(TextNode { text: text.into() }),
        }
    }

    /// Create a text input node
    pub fn text_input(id: impl Into<NodeId>, label: impl Into<String>) -> TextInputBuilder {
        TextInputBuilder::new(id.into(), label.into())
    }

    // Common options setters (builder pattern)

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
}
