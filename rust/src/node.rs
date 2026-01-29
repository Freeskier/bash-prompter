use crossterm::style::Color;

/// Options for configuring a node's rendering behavior
#[derive(Debug, Clone)]
pub struct NodeOptions {
    /// Foreground color
    pub fg_color: Option<Color>,
    /// Background color
    pub bg_color: Option<Color>,
    /// Whether this node is sticky (position relative to viewport)
    pub sticky: bool,
    /// Whether to start on a new line before rendering
    pub new_line: bool,
}

impl Default for NodeOptions {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            sticky: false,
            new_line: false,
        }
    }
}

impl NodeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.fg_color = Some(color);
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn with_colors(mut self, fg: Color, bg: Color) -> Self {
        self.fg_color = Some(fg);
        self.bg_color = Some(bg);
        self
    }

    pub fn with_sticky(mut self, sticky: bool) -> Self {
        self.sticky = sticky;
        self
    }

    pub fn with_new_line(mut self, new_line: bool) -> Self {
        self.new_line = new_line;
        self
    }
}

/// Base trait for all renderable nodes
pub trait Node {
    /// Get node options
    fn options(&self) -> &NodeOptions;

    /// Get mutable node options
    fn options_mut(&mut self) -> &mut NodeOptions;

    /// Get the content to render as lines
    /// Each string in the vec is one line
    fn content(&self) -> Vec<String>;

    /// Get the actual rendered position (set by renderer)
    fn position(&self) -> (u16, u16);

    /// Set the actual rendered position (called by renderer)
    fn set_position(&mut self, x: u16, y: u16);

    /// Get width (calculated from content)
    fn width(&self) -> u16;

    /// Get height (number of lines)
    fn height(&self) -> u16;
}

/// Text node - renders static text
pub struct TextNode {
    options: NodeOptions,
    text: String,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl TextNode {
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let lines = text.split('\n').collect::<Vec<_>>();
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        let height = lines.len() as u16;

        Self {
            options: NodeOptions::default(),
            text,
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    pub fn with_options(mut self, options: NodeOptions) -> Self {
        self.options = options;
        self
    }

    pub fn with_fg_color(mut self, color: Color) -> Self {
        self.options.fg_color = Some(color);
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.options.bg_color = Some(color);
        self
    }

    pub fn with_colors(mut self, fg: Color, bg: Color) -> Self {
        self.options.fg_color = Some(fg);
        self.options.bg_color = Some(bg);
        self
    }

    pub fn with_sticky(mut self, sticky: bool) -> Self {
        self.options.sticky = sticky;
        self
    }

    pub fn with_new_line(mut self, new_line: bool) -> Self {
        self.options.new_line = new_line;
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        // Recalculate width and height
        let lines = self.text.split('\n').collect::<Vec<_>>();
        self.width = lines.iter().map(|line| line.len()).max().unwrap_or(0) as u16;
        self.height = lines.len() as u16;
    }
}

impl Node for TextNode {
    fn options(&self) -> &NodeOptions {
        &self.options
    }

    fn options_mut(&mut self) -> &mut NodeOptions {
        &mut self.options
    }

    fn content(&self) -> Vec<String> {
        self.text.split('\n').map(|s| s.to_string()).collect()
    }

    fn position(&self) -> (u16, u16) {
        (self.x, self.y)
    }

    fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }
}
