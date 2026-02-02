use crate::input::{Input, NodeId};
use crate::node::Node;

pub struct FormStep {
    pub prompt: String,
    pub hint: Option<String>,
    pub nodes: Vec<Node>,
}

pub trait FormStepExt {
    fn find_input(&self, id: &str) -> Option<&dyn Input>;
    fn find_input_mut(&mut self, id: &str) -> Option<&mut dyn Input>;
    fn validate_all(&self) -> Vec<(NodeId, String)>;
    fn values(&self) -> Vec<(NodeId, String)>;
}

impl FormStepExt for FormStep {
    fn find_input(&self, id: &str) -> Option<&dyn Input> {
        self.nodes
            .iter()
            .filter_map(|node| node.as_input())
            .find(|input| input.id() == id)
    }

    fn find_input_mut(&mut self, id: &str) -> Option<&mut dyn Input> {
        self.nodes
            .iter_mut()
            .filter_map(|node| node.as_input_mut())
            .find(|input| input.id() == id)
    }

    fn validate_all(&self) -> Vec<(NodeId, String)> {
        self.nodes
            .iter()
            .filter_map(|node| node.as_input())
            .filter_map(|input| input.validate().err().map(|err| (input.id().clone(), err)))
            .collect()
    }

    fn values(&self) -> Vec<(NodeId, String)> {
        self.nodes
            .iter()
            .filter_map(|node| node.as_input())
            .map(|input| (input.id().clone(), input.value()))
            .collect()
    }
}
