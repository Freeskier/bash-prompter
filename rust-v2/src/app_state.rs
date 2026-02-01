use crate::node::NodeId;
use crate::step::Step;
use std::collections::HashMap;

/// AppState manages UI state - focus, input order, and O(1) lookup
#[derive(Debug, Clone)]
pub struct AppState {
    /// Currently focused input ID
    focused: Option<NodeId>,

    /// Ordered list of all input IDs (for Tab navigation)
    input_order: Vec<NodeId>,

    /// Map from NodeId to Step index for O(1) lookup
    input_index: HashMap<NodeId, usize>,
}

impl AppState {
    /// Create new empty state
    pub fn new() -> Self {
        Self {
            focused: None,
            input_order: Vec::new(),
            input_index: HashMap::new(),
        }
    }

    /// Rebuild state from a Step (call after Step changes)
    pub fn from_step(step: &Step) -> Self {
        let mut order = Vec::new();
        let mut index = HashMap::new();

        for (step_idx, node) in step.iter().enumerate() {
            if let Some(id) = node.kind.input_id() {
                order.push(id.clone());
                index.insert(id.clone(), step_idx);
            }
        }

        // Auto-focus first input
        let focused = order.first().cloned();

        Self {
            focused,
            input_order: order,
            input_index: index,
        }
    }

    /// Get currently focused input ID
    pub fn focused(&self) -> Option<&NodeId> {
        self.focused.as_ref()
    }

    /// Get Step index of focused input (O(1))
    pub fn focused_index(&self) -> Option<usize> {
        let id = self.focused.as_ref()?;
        self.input_index.get(id).copied()
    }

    /// Get Step index for a specific input ID (O(1))
    pub fn index_of(&self, id: &NodeId) -> Option<usize> {
        self.input_index.get(id).copied()
    }

    /// Focus next input (wraps around)
    pub fn focus_next(&mut self) {
        if self.input_order.is_empty() {
            self.focused = None;
            return;
        }

        match &self.focused {
            None => self.focused = self.input_order.first().cloned(),
            Some(current) => {
                let pos = self
                    .input_order
                    .iter()
                    .position(|id| id == current)
                    .unwrap_or(0);

                let next_pos = (pos + 1) % self.input_order.len();
                self.focused = Some(self.input_order[next_pos].clone());
            }
        }
    }

    /// Focus previous input (wraps around)
    pub fn focus_prev(&mut self) {
        if self.input_order.is_empty() {
            self.focused = None;
            return;
        }

        match &self.focused {
            None => self.focused = self.input_order.first().cloned(),
            Some(current) => {
                let pos = self
                    .input_order
                    .iter()
                    .position(|id| id == current)
                    .unwrap_or(0);

                let prev_pos = if pos == 0 {
                    self.input_order.len() - 1
                } else {
                    pos - 1
                };

                self.focused = Some(self.input_order[prev_pos].clone());
            }
        }
    }

    /// Focus specific input by ID
    pub fn focus(&mut self, id: &NodeId) {
        if self.input_index.contains_key(id) {
            self.focused = Some(id.clone());
        }
    }

    /// Check if a specific input is focused
    pub fn is_focused(&self, id: &NodeId) -> bool {
        self.focused.as_ref() == Some(id)
    }

    /// Get list of all input IDs in order
    pub fn input_ids(&self) -> &[NodeId] {
        &self.input_order
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
