use crate::Error;

use nalgebra::{vector, Vector3};

use alloc::boxed::Box;
use core::{fmt::Debug, ops::Deref};

pub(crate) const OCTREE_CHILDREN: usize = 8;

const BOUNDS_LEN: usize = 2;

#[repr(usize)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Octant {
    LeftRearBase = 0,
    RightRearBase = 1,
    LeftRearTop = 2,
    RightRearTop = 3,
    LeftFrontBase = 4,
    RightFrontBase = 5,
    LeftFrontTop = 6,
    RightFrontTop = 7,
}

impl Octant {
    fn offset(&self) -> Vector3<u32> {
        match self {
            Self::LeftRearBase => vector![0, 0, 0],
            Self::RightRearBase => vector![1, 0, 0],
            Self::LeftRearTop => vector![0, 0, 1],
            Self::RightRearTop => vector![1, 0, 1],
            Self::LeftFrontBase => vector![0, 1, 0],
            Self::RightFrontBase => vector![1, 1, 0],
            Self::LeftFrontTop => vector![0, 1, 1],
            Self::RightFrontTop => vector![1, 1, 1],
        }
    }

    fn vector_diff(rhs: Vector3<u32>, lhs: Vector3<u32>) -> Self {
        if lhs.z < rhs.z {
            if lhs.y < rhs.y {
                if lhs.x < rhs.x {
                    Self::LeftRearBase
                } else {
                    Self::RightRearBase
                }
            } else {
                if lhs.x < rhs.x {
                    Self::LeftFrontBase
                } else {
                    Self::RightFrontBase
                }
            }
        } else {
            if lhs.y < rhs.y {
                if lhs.x < rhs.x {
                    Self::LeftRearTop
                } else {
                    Self::RightRearTop
                }
            } else {
                if lhs.x < rhs.x {
                    Self::LeftFrontTop
                } else {
                    Self::RightFrontTop
                }
            }
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

#[derive(Debug, Default, Clone)]
pub(crate) struct Node<T> {
    ty: NodeType<T>,
    bounds: [Vector3<u32>; BOUNDS_LEN],
    children: [Box<Option<Node<T>>>; OCTREE_CHILDREN],
}

impl<T: Debug + Default + Eq + PartialEq + Clone> Node<T> {
    /// Creates a new `Node<T>` with the given bounds.
    pub(crate) fn new(bounds: [Vector3<u32>; BOUNDS_LEN]) -> Self {
        Self {
            ty: NodeType::Internal,
            bounds,
            ..Default::default()
        }
    }

    /// Inserts a new leaf `Node` at the given position, if possible.
    pub(crate) fn insert(&mut self, position: Vector3<u32>, data: T) -> Result<(), Error> {
        if self.contains(position) {
            if self.dimension() == 1 {
                self.ty = NodeType::Leaf(data);
            } else {
                self.ty = NodeType::Internal;

                let dimension = self.dimension() / 2;
                let dimension_3d = vector![dimension, dimension, dimension];
                let midpoint = self.min_position() + dimension_3d;
                let octant = Octant::vector_diff(midpoint, position);

                let lower = self.min_position() + dimension_3d.component_mul(&octant.offset());
                let upper = lower + dimension_3d;
                let bounds = [lower, upper];

                let mut node = if self.children[octant as usize].as_ref().is_some() {
                    self.children[octant as usize].take().unwrap()
                } else {
                    Node::<T>::new(bounds)
                };

                node.insert(position, data).unwrap();

                self.children[octant as usize] = Box::new(Some(node));
            }

            self.simplify();
            return Ok(());
        }

        Err(Error::InvalidPosition(position))
    }

    /// Gets data from a `Node` at the given position, if possible.
    pub(crate) fn get(&self, position: Vector3<u32>) -> Option<&T> {
        if self.contains(position) {
            return match &self.ty {
                NodeType::Leaf(data) => Some(data),
                _ => {
                    let dimension = self.dimension() / 2;
                    let dimension_3d = vector![dimension, dimension, dimension];
                    let midpoint = self.min_position() + dimension_3d;
                    let octant = Octant::vector_diff(midpoint, position);

                    match self.children[octant as usize].deref() {
                        Some(child) => child.get(position),
                        _ => None,
                    }
                }
            };
        }

        None
    }

    /// Simplifies the `Node`.
    ///
    /// If all children are leaf `Node`s with identical data, destroy all children, 
    /// and mark the `Node` as a leaf containing that data.
    pub(crate) fn simplify(&mut self) {
        let mut data = None;

        for i in 0..OCTREE_CHILDREN {
            if let Some(child) = self.children[i].deref() {
                if child.is_leaf() {
                    let leaf_data = child.leaf_data();

                    if leaf_data.is_none() {
                        return;
                    } else if data.as_ref().is_none() {
                        data = leaf_data;
                    } else if *data.as_ref().unwrap() != leaf_data.unwrap() {
                        return;
                    }
                }
            } else {
                return;
            }
        }

        self.ty = NodeType::Leaf((*data.unwrap()).clone());

        for i in 0..OCTREE_CHILDREN {
            *self.children[i] = None;
        }
    }

    /// Returns the dimension of the `Node`.
    pub(crate) fn dimension(&self) -> u32 {
        (self.bounds[0].x as i32 - self.bounds[1].x as i32).abs() as u32
    }

    /// Returns whether the `Node` contains the given position.
    pub(crate) fn contains(&self, position: Vector3<u32>) -> bool {
        position.x >= self.bounds[0].x
            && position.x < self.bounds[1].x
            && position.y >= self.bounds[0].y
            && position.y < self.bounds[1].y
            && position.z >= self.bounds[0].z
            && position.z < self.bounds[1].z
    }

    /// Get leaf data from this `Node`.
    pub(crate) fn leaf_data(&self) -> Option<&T> {
        match &self.ty {
            NodeType::Leaf(data) => Some(&data),
            _ => None,
        }
    }

    fn min_position(&self) -> Vector3<u32> {
        self.bounds[0]
    }

    fn is_leaf(&self) -> bool {
        matches!(self.ty, NodeType::Leaf(_))
    }
}
