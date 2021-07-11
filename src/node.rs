use nalgebra::{Vector3, vector};

use std::ops::{Deref, DerefMut};

pub(crate) const OCTREE_CHILDREN: usize = 8;

const BOUNDS_LEN: usize = 2;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Vertex {
    FrontLowerLeft  = 0,
    FrontLowerRight = 1,
    FrontUpperLeft  = 2,
    FrontUpperRight = 3,
    BackLowerLeft   = 4,
    BackLowerRight  = 5,
    BackUpperLeft   = 6,
    BackUpperRight  = 7,
}

impl Vertex {
    fn offset(&self) -> Vector3<u32> {
        match self {
            Self::FrontLowerLeft  => vector![0, 0, 0],
            Self::FrontLowerRight => vector![0, 1, 0],
            Self::FrontUpperLeft  => vector![0, 0, 1],
            Self::FrontUpperRight => vector![0, 1, 1],
            Self::BackLowerLeft   => vector![1, 0, 0],
            Self::BackLowerRight  => vector![1, 1, 0],
            Self::BackUpperLeft   => vector![1, 0, 1],
            Self::BackUpperRight  => vector![1, 1, 1],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum NodeType<T> {
    Leaf(T),
    Internal,
}

impl<T> Default for NodeType<T> {
    fn default() -> Self {
        Self::Internal
    }
}

#[derive(Debug, Default)]
pub(crate) struct Node<T> {
    ty: NodeType<T>,
    bounds: [Vector3<u32>; BOUNDS_LEN],
    children: [Box<Option<Node<T>>>; OCTREE_CHILDREN],
}

impl<T: Default> Node<T> {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn child(&self, index: usize) -> Option<&Node<T>> {
        match self.children.get(index) {
            Some(child) => child.deref().as_ref(),
            _ => None,
        }
    }

    pub fn child_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        match self.children.get_mut(index) {
            Some(child) => child.deref_mut().as_mut(),
            _ => None,
        }
    }

    pub fn insert_child(&mut self, index: usize, node: Node<T>) -> Result<(), ()> {
        if index > OCTREE_CHILDREN {
            return Err(());
        }

        self.children[index] = Box::new(Some(node));
        Ok(())
    }

    pub fn get_leaf_data(&self) -> Option<&T> {
        match &self.ty {
            NodeType::Leaf(data) => Some(&data),
            _ => None,
        }
    }

    pub fn set_leaf_data(&mut self, data: T) -> Result<(), ()> {
        if !self.is_leaf() {
            return Err(());
        }
        
        self.ty = NodeType::Leaf(data);
        Ok(())
    }

    pub fn dimension(&self) -> u32 {
        (self.bounds[0].x as i32 - self.bounds[1].x as i32).abs() as u32
    }

    pub fn vertex_position(&self, position: Vertex) -> Vector3<u32> {
        self.bounds[0] + position.offset()
    }

    pub fn is_leaf(&self) -> bool {
        match self.ty {
            NodeType::Leaf(_) => true,
            _ => false,
        }
    }
}
