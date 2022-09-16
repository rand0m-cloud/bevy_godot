use crate::prelude::Node;

pub trait NodeTreeView {
    fn from_node(node: &Node) -> Self;
}
