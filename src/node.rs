use nalgebra::{vector, Vector3};

use std::{fmt::Debug, ops::Deref};

pub(crate) const OCTREE_CHILDREN: usize = 8;

const BOUNDS_LEN: usize = 2;

#[repr(usize)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Default)]
pub(crate) struct Node<T> {
    ty: NodeType<T>,
    bounds: [Vector3<u32>; BOUNDS_LEN],
    children: [Box<Option<Node<T>>>; OCTREE_CHILDREN],
}

impl<T: Debug + Default> Node<T> {
    /// Creates a new `Node<T>` with the given bounds.
    pub(crate) fn new(bounds: [Vector3<u32>; BOUNDS_LEN]) -> Self {
        Self {
            ty: NodeType::Internal,
            bounds,
            ..Default::default()
        }
    }

    /// Inserts a new leaf `Node` at the given position, if possible.
    pub fn insert(&mut self, position: Vector3<u32>, data: T) -> Result<(), ()> {
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

                let mut node = Node::<T>::new(bounds);
                node.insert(position, data).unwrap();

                self.children[octant as usize] = Box::new(Some(node));
            }

            return Ok(());
        }

        Err(())
    }

    /// Gets data from a `Node` at the given position, if possible.
    pub fn get(&self, position: Vector3<u32>) -> Option<&T> {
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

    /// Returns the dimension of the `Node`.
    pub fn dimension(&self) -> u32 {
        (self.bounds[0].x as i32 - self.bounds[1].x as i32).abs() as u32
    }

    /// Returns the position at the minimum point of the `Node`.
    pub fn min_position(&self) -> Vector3<u32> {
        self.bounds[0]
    }

    /// Returns whether the `Node` contains the given position.
    pub fn contains(&self, position: Vector3<u32>) -> bool {
        position.x >= self.bounds[0].x
            && position.x < self.bounds[1].x
            && position.y >= self.bounds[0].y
            && position.y < self.bounds[1].y
            && position.z >= self.bounds[0].z
            && position.z < self.bounds[1].z
    }

    fn is_leaf(&self) -> bool {
        match self.ty {
            NodeType::Leaf(_) => true,
            _ => false,
        }
    }
}
