use crate::{Error, Node};

use nalgebra::{vector, Vector3};

use alloc::boxed::Box;
use core::{fmt::Debug, num::NonZeroU32};

#[derive(Debug)]
pub struct Octree<T> {
    root: Box<Node<T>>,
}

impl<T: Debug + Default + Eq + PartialEq + Clone> Octree<T> {
    /// Creates a new `Octree<T>` of given dimension.
    /// Returns an error if the dimension is 0
    pub fn new(dimension: NonZeroU32) -> Result<Self, Error> {
        // Check that `dimension` is a power of 2.
        if (dimension.get() as f32).log(2.0).fract() == 0.0 {
            return Ok(Self {
                root: Box::new(Node::<T>::new([
                    vector![0, 0, 0],
                    vector![dimension.get(), dimension.get(), dimension.get()],
                ])),
            });
        }

        Err(Error::InvalidDimension(dimension))
    }

    /// Inserts data of type `T` into the given position in the `Octree`.
    /// Returns an error if the position does not exist within the confines of the `Octree`.
    pub fn insert(&mut self, position: Vector3<u32>, data: T) -> Result<(), Error> {
        self.root.insert(position, data)
    }

    /// Retrieves data of type `T` from the given position in the `Octree`.
    /// Since the `Octree` is sparse, returns `None` if the position does not currently store any data.
    pub fn get(&self, position: Vector3<u32>) -> Option<&T> {
        self.root.get(position)
    }

    /// Returns the dimension of the root node.
    pub fn dimension(&self) -> u32 {
        self.root.dimension()
    }

    /// Returns whether the given position exists within the confines of the `Octree`.
    pub fn contains(&self, position: Vector3<u32>) -> bool {
        self.root.contains(position)
    }
}
