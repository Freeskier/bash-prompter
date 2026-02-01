use crate::node::{Node, NodeId};
use std::collections::HashMap;

/// A Step is a collection of nodes (like a form or a screen)
pub type Step = Vec<Node>;

/// Validation error for a specific input
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub id: NodeId,
    pub message: String,
}

/// Extension trait for Step to add helper methods
pub trait StepExt {
    /// Find node by ID (O(n) - use AppState index for O(1) access)
    fn find_by_id(&self, id: &NodeId) -> Option<&Node>;

    /// Find node by ID (mutable)
    fn find_by_id_mut(&mut self, id: &NodeId) -> Option<&mut Node>;

    /// Get value of input by ID
    fn get_value(&self, id: &NodeId) -> Option<&str>;

    /// Set value of input by ID
    fn set_value(&mut self, id: &NodeId, value: &str) -> bool;

    /// Append text to input by ID
    fn append_value(&mut self, id: &NodeId, text: &str) -> bool;

    /// Insert text to input by ID (uses InputNode method)
    fn insert_text(&mut self, id: &NodeId, text: &str) -> bool;

    /// Delete last character from input by ID
    fn delete_char(&mut self, id: &NodeId) -> bool;

    /// Delete character forward (Delete key)
    fn delete_char_forward(&mut self, id: &NodeId) -> bool;

    /// Delete last word from input by ID (Ctrl+Backspace)
    fn delete_word(&mut self, id: &NodeId) -> bool;

    /// Move cursor left in input
    fn move_cursor_left(&mut self, id: &NodeId) -> bool;

    /// Move cursor right in input
    fn move_cursor_right(&mut self, id: &NodeId) -> bool;

    /// Move cursor to start (Home)
    fn move_cursor_home(&mut self, id: &NodeId) -> bool;

    /// Move cursor to end (End)
    fn move_cursor_end(&mut self, id: &NodeId) -> bool;

    /// Clear input value by ID
    fn clear_input(&mut self, id: &NodeId) -> bool;

    /// Validate all inputs in the step
    fn validate_all(&self) -> Vec<ValidationError>;

    /// Validate specific input by ID
    fn validate_input(&self, id: &NodeId) -> Result<(), String>;

    /// Set error message for input by ID
    fn set_error(&mut self, id: &NodeId, error: String) -> bool;

    /// Clear error message for input by ID
    fn clear_error(&mut self, id: &NodeId) -> bool;

    /// Get all input values as HashMap
    fn values(&self) -> HashMap<NodeId, String>;
}

impl StepExt for Step {
    fn find_by_id(&self, id: &NodeId) -> Option<&Node> {
        self.iter().find(|node| node.kind.input_id() == Some(id))
    }

    fn find_by_id_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.iter_mut()
            .find(|node| node.kind.input_id() == Some(id))
    }

    fn get_value(&self, id: &NodeId) -> Option<&str> {
        self.find_by_id(id)?.kind.value()
    }

    fn set_value(&mut self, id: &NodeId, value: &str) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.value_mut())
            .map(|v| {
                *v = value.to_string();
                true
            })
            .unwrap_or(false)
    }

    fn append_value(&mut self, id: &NodeId, text: &str) -> bool {
        if let Some(node) = self.find_by_id_mut(id)
            && let Some(v) = node.kind.value_mut() {
                v.push_str(text);
                return true;
            }
        false
    }

    fn insert_text(&mut self, id: &NodeId, text: &str) -> bool {
        if let Some(node) = self.find_by_id_mut(id)
            && let Some(input) = node.kind.input_mut() {
                input.insert_text(text);
                return true;
            }
        false
    }

    fn delete_char(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.delete_char())
            .unwrap_or(false)
    }

    fn delete_char_forward(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.delete_char_forward())
            .unwrap_or(false)
    }

    fn delete_word(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.delete_word())
            .unwrap_or(false)
    }

    fn move_cursor_left(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.move_cursor_left())
            .unwrap_or(false)
    }

    fn move_cursor_right(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.move_cursor_right())
            .unwrap_or(false)
    }

    fn move_cursor_home(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.move_cursor_home())
            .unwrap_or(false)
    }

    fn move_cursor_end(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| input.move_cursor_end())
            .unwrap_or(false)
    }

    fn clear_input(&mut self, id: &NodeId) -> bool {
        self.find_by_id_mut(id)
            .and_then(|node| node.kind.input_mut())
            .map(|input| {
                input.clear();
                true
            })
            .unwrap_or(false)
    }

    fn validate_all(&self) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for node in self {
            if let Some(input) = node.kind.input()
                && let Err(msg) = input.validate() {
                    errors.push(ValidationError {
                        id: input.id.clone(),
                        message: msg,
                    });
                }
        }

        errors
    }

    fn validate_input(&self, id: &NodeId) -> Result<(), String> {
        let node = self
            .find_by_id(id)
            .ok_or_else(|| "Input not found".to_string())?;

        if let Some(input) = node.kind.input() {
            input.validate()
        } else {
            Ok(())
        }
    }

    fn set_error(&mut self, id: &NodeId, error: String) -> bool {
        if let Some(node) = self.find_by_id_mut(id)
            && let Some(input) = node.kind.input_mut() {
                input.error = Some(error);
                return true;
            }
        false
    }

    fn clear_error(&mut self, id: &NodeId) -> bool {
        if let Some(node) = self.find_by_id_mut(id)
            && let Some(input) = node.kind.input_mut() {
                input.error = None;
                return true;
            }
        false
    }

    fn values(&self) -> HashMap<NodeId, String> {
        self.iter()
            .filter_map(|node| {
                let id = node.kind.input_id()?;
                let value = node.kind.value()?;
                Some((id.clone(), value.to_string()))
            })
            .collect()
    }
}
