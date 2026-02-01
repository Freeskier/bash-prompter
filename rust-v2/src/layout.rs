use crate::node::{Display, Wrap};
use crate::node::{Node, NodeKind};
use unicode_width::UnicodeWidthStr;

/// Positioned node - contains the index and position for a node
#[derive(Debug, Clone)]
pub struct Placed {
    /// Index in the Step (Vec<Node>)
    pub index: usize,
    /// X position (column)
    pub x: usize,
    /// Y position (row/line)
    pub y: usize,
}

/// Layout engine - converts Step (&[Node]) into positioned nodes (Vec<Placed>)
#[derive(Debug, Clone)]
pub struct Layout {
    margin: usize,
}

impl Layout {
    pub fn new() -> Self {
        Self { margin: 2 }
    }

    pub fn with_margin(mut self, margin: usize) -> Self {
        self.margin = margin;
        self
    }

    /// Layout nodes and return their positions
    pub fn layout(&self, nodes: &[Node], width: u16) -> Vec<Placed> {
        let effective_width = (width as usize).saturating_sub(self.margin);
        let mut engine = LayoutEngine::new(effective_width);

        for (index, node) in nodes.iter().enumerate() {
            engine.place_node(index, node);
        }

        engine.finish()
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal layout engine
struct LayoutEngine {
    width: usize,
    current_x: usize,
    current_y: usize,
    placements: Vec<Placed>,
}

impl LayoutEngine {
    fn new(width: usize) -> Self {
        Self {
            width,
            current_x: 0,
            current_y: 0,
            placements: Vec::new(),
        }
    }

    fn place_node(&mut self, index: usize, node: &Node) {
        // Block element starts on a new line if something is already on the current line
        if node.opts.display == Display::Block && self.current_x > 0 {
            self.new_line();
        }

        // Calculate the width of this node
        let node_width = self.estimate_node_width(node);

        // Handle wrapping
        match node.opts.wrap {
            Wrap::No => {
                // No wrap: if doesn't fit on current line, move to next line
                if self.current_x > 0 && node_width > self.available_width() {
                    self.new_line();
                }

                // Place the node (even if it exceeds width - it will be clipped)
                self.place_at_current(index);
                self.current_x += node_width.min(self.width);
            }
            Wrap::Yes => {
                // With wrap: place on current line (wrapping will be handled during rendering)
                // For now, just track that we placed it
                if self.current_x >= self.width {
                    self.new_line();
                }

                self.place_at_current(index);

                // Update position based on content
                // If wrapping is needed, renderer will handle it
                self.current_x += node_width;

                // If we exceeded the width, move to next line for next node
                if self.current_x >= self.width {
                    self.new_line();
                }
            }
        }

        // Block elements don't automatically end the line
        // The next element decides whether to continue or start new line
    }

    fn place_at_current(&mut self, index: usize) {
        self.placements.push(Placed {
            index,
            x: self.current_x,
            y: self.current_y,
        });
    }

    fn new_line(&mut self) {
        self.current_y += 1;
        self.current_x = 0;
    }

    fn available_width(&self) -> usize {
        self.width.saturating_sub(self.current_x)
    }

    /// Estimate the display width of a node
    fn estimate_node_width(&self, node: &Node) -> usize {
        match &node.kind {
            NodeKind::Text(text_node) => {
                // Simple text: just the text width
                text_node.text.width()
            }
            NodeKind::TextInput(text_input) => {
                // TextInput format: "label: value" or "label: [value]" if focused
                // For layout purposes, we need to know if it's focused
                // Since we don't have focus info here, we estimate max width

                let input = &text_input.input;
                let label_width = input.label.width();
                let colon_space = ": ".width(); // 2
                let value_width = input.value.width();

                // Use the larger of: min_width or actual value width
                let content_width = input.min_width.max(value_width);

                // Assume brackets for focused state (worst case)
                let brackets = "[]".width(); // 2

                label_width + colon_space + content_width + brackets
            }
        }
    }

    fn finish(self) -> Vec<Placed> {
        self.placements
    }
}
