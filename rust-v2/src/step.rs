use crate::input::{Input, NodeId};
use crate::node::Node;

pub type Step = Vec<Node>;

pub trait StepHelpers {
    fn find_input(&self, id: &str) -> Option<&dyn Input>;
    fn find_input_mut(&mut self, id: &str) -> Option<&mut dyn Input>;
    fn validate_all(&self) -> Vec<(NodeId, String)>;
    fn values(&self) -> Vec<(NodeId, String)>;
}

impl StepHelpers for Step {
    fn find_input(&self, id: &str) -> Option<&dyn Input> {
        self.iter()
            .filter_map(|node| node.as_input())
            .find(|input| input.id() == id)
    }

    fn find_input_mut(&mut self, id: &str) -> Option<&mut dyn Input> {
        self.iter_mut()
            .filter_map(|node| node.as_input_mut())
            .find(|input| input.id() == id)
    }

    fn validate_all(&self) -> Vec<(NodeId, String)> {
        self.iter()
            .filter_map(|node| node.as_input())
            .filter_map(|input| input.validate().err().map(|err| (input.id().clone(), err)))
            .collect()
    }

    fn values(&self) -> Vec<(NodeId, String)> {
        self.iter()
            .filter_map(|node| node.as_input())
            .map(|input| (input.id().clone(), input.value()))
            .collect()
    }
}
