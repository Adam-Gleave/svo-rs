use crate::Node;

#[derive(Debug)]
pub struct Octree<T> {
    root: Box<Node<T>>,
}
