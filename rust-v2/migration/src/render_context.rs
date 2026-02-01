use crate::node::NodeId;

/// RenderContext holds information needed during rendering
/// - Who is focused
/// - Any other rendering hints
#[derive(Debug, Clone)]
pub struct RenderContext<'a> {
    /// Currently focused input ID
    pub focused_id: Option<&'a NodeId>,
}

impl<'a> RenderContext<'a> {
    pub fn new(focused_id: Option<&'a NodeId>) -> Self {
        Self { focused_id }
    }

    /// Check if a specific ID is focused
    pub fn is_focused(&self, id: &NodeId) -> bool {
        self.focused_id == Some(id)
    }
}
