use crate::prelude::{godot_prelude::SubClass, Node, TRef};

pub trait NodeTreeView {
    fn from_node<T: SubClass<Node>>(node: TRef<T>) -> Self;
}
