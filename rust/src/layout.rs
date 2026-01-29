use crate::node::Node;
use crate::renderer::Renderer;
use anyhow::Result;

/// Layout manages a collection of nodes and calculates their positions
pub struct Layout {
    nodes: Vec<Box<dyn Node>>,
    focused_index: Option<usize>,
    start_row: u16,
}

impl Layout {
    /// Create a new layout starting at given row
    pub fn new(start_row: u16) -> Self {
        Self {
            nodes: Vec::new(),
            focused_index: None,
            start_row,
        }
    }

    /// Add a node to the layout and calculate its position
    pub fn add(&mut self, mut node: Box<dyn Node>) {
        let (x, y) = self.calculate_next_position(&node);
        node.set_position(x, y);
        self.nodes.push(node);
    }

    /// Calculate position for the next node based on previous nodes
    fn calculate_next_position(&self, node: &Box<dyn Node>) -> (u16, u16) {
        let options = node.options();

        // If new_line is set, start at beginning of next line
        if options.new_line {
            if let Some(last_node) = self.nodes.last() {
                let (_, last_y) = last_node.position();
                let last_height = last_node.height();
                return (0, last_y + last_height);
            } else {
                return (0, self.start_row);
            }
        }

        // Otherwise, continue inline from last node
        if let Some(last_node) = self.nodes.last() {
            let (last_x, last_y) = last_node.position();
            let last_width = last_node.width();

            // Check if last node was multi-line
            let last_height = last_node.height();
            if last_height > 1 {
                // Multi-line node - start at beginning of line after it
                (0, last_y + last_height)
            } else {
                // Single line - continue inline
                (last_x + last_width, last_y)
            }
        } else {
            // First node
            (0, self.start_row)
        }
    }

    /// Get reference to all nodes
    pub fn nodes(&self) -> &[Box<dyn Node>] {
        &self.nodes
    }

    /// Get mutable reference to all nodes
    pub fn nodes_mut(&mut self) -> &mut [Box<dyn Node>] {
        &mut self.nodes
    }

    /// Get reference to a specific node
    pub fn get(&self, index: usize) -> Option<&Box<dyn Node>> {
        self.nodes.get(index)
    }

    /// Get mutable reference to a specific node
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Box<dyn Node>> {
        self.nodes.get_mut(index)
    }

    /// Set which node has focus (for interactive nodes)
    pub fn set_focus(&mut self, index: Option<usize>) {
        self.focused_index = index;
    }

    /// Get currently focused node index
    pub fn focused_index(&self) -> Option<usize> {
        self.focused_index
    }

    /// Get mutable reference to focused node
    pub fn focused_node_mut(&mut self) -> Option<&mut Box<dyn Node>> {
        self.focused_index.and_then(|idx| self.nodes.get_mut(idx))
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.focused_index = None;
    }

    /// Recalculate all positions (e.g., after terminal resize or content change)
    pub fn recalculate_positions(&mut self) {
        let mut current_x = 0u16;
        let mut current_y = self.start_row;

        for node in &mut self.nodes {
            let options = node.options();

            if options.new_line || current_x == 0 {
                // Start on new line
                if current_x != 0 {
                    current_y += 1;
                }
                current_x = 0;
            }

            node.set_position(current_x, current_y);

            let width = node.width();
            let height = node.height();

            if height > 1 {
                // Multi-line node
                current_y += height;
                current_x = 0;
            } else {
                // Single line - move cursor
                current_x += width;
            }
        }
    }

    /// Get the total height occupied by all nodes
    pub fn total_height(&self) -> u16 {
        if let Some(last_node) = self.nodes.last() {
            let (_, last_y) = last_node.position();
            let last_height = last_node.height();
            (last_y + last_height) - self.start_row
        } else {
            0
        }
    }

    /// Render all nodes using the renderer
    pub fn render(&self, renderer: &mut Renderer) -> Result<()> {
        let term_height = renderer.context().height();

        // First, clear all lines that will be used (only visible ones)
        let max_y = self
            .nodes
            .iter()
            .map(|node| {
                let (_, y) = node.position();
                y + node.height()
            })
            .max()
            .unwrap_or(self.start_row);

        for y in self.start_row..max_y.min(term_height) {
            renderer.move_to(0, y)?;
            renderer.clear_line()?;
        }

        // Now render all nodes (skip those above or below viewport)
        for node in &self.nodes {
            let (x, y) = node.position();
            let content = node.content();

            for (line_idx, line) in content.iter().enumerate() {
                let line_y = y + line_idx as u16;

                // Skip rendering if line is outside viewport
                if line_y >= term_height {
                    break;
                }

                // Render with colors if available
                let options = node.options();
                if let (Some(fg), Some(bg)) = (options.fg_color, options.bg_color) {
                    renderer.draw_colored_at(x, line_y, line, fg, Some(bg))?;
                } else if let Some(fg) = options.fg_color {
                    renderer.draw_colored_at(x, line_y, line, fg, None)?;
                } else {
                    renderer.draw_at(x, line_y, line)?;
                }
            }
        }

        renderer.flush()?;
        Ok(())
    }

    /// Update start row (e.g., after scroll)
    pub fn set_start_row(&mut self, row: u16) {
        let offset = (row as i32) - (self.start_row as i32);
        self.start_row = row;

        // Update all node positions
        for node in &mut self.nodes {
            let (x, y) = node.position();
            let new_y = ((y as i32) + offset).max(0) as u16;
            node.set_position(x, new_y);
        }
    }
}
