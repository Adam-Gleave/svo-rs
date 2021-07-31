use crate::{Error, Node, Vector3};

#[cfg(feature = "no-std")]
use micromath::F32Ext;

use alloc::boxed::Box;
use core::{f32, fmt::Debug, hash::Hash, num::NonZeroU32};

#[derive(Debug)]
pub struct Octree<T>
where
    T: Debug + Default + Clone + Eq + PartialEq + Ord + PartialOrd + Copy + Hash,
{
    dimension: NonZeroU32,
    root: Box<Node<T>>,
}

impl<T> Octree<T>
where
    T: Debug + Default + Clone + Eq + PartialEq + Ord + PartialOrd + Copy + Hash,
{
    /// Creates a new `Octree<T>` of given dimension.
    /// Returns an error if the dimension is 0
    pub fn new(dimension: NonZeroU32) -> Result<Self, Error> {
        // Check that `dimension` is a power of 2.
        if (dimension.get() as f32).log(2.0).fract() == 0.0 {
            return Ok(Self {
                dimension,
                root: Box::new(Node::<T>::new([
                    Vector3::from([0, 0, 0]),
                    Vector3::from([dimension.get(), dimension.get(), dimension.get()]),
                ])),
            });
        }

        Err(Error::InvalidDimension(dimension))
    }

    /// Inserts data of type `T` into the given position in the `Octree`.
    /// Returns an error if the position does not exist within the confines of the `Octree`.
    pub fn insert(&mut self, position: [u32; 3], data: T) -> Result<(), Error> {
        self.root.insert(position.into(), data)
    }

    /// Retrieves data of type `T` from the given position in the `Octree`.
    /// Since the `Octree` is sparse, returns `None` if the position does not currently store any data.
    pub fn get(&self, position: [u32; 3]) -> Option<&T> {
        self.root.get(position.into())
    }

    /// Removes the `Node` at the given position in the `Octree`, if it exists.
    /// This will simplify the `Octree` if `auto_simplify` is specified.
    pub fn clear_at(&mut self, position: [u32; 3]) -> Result<(), Error> {
        self.root.clear(position.into())
    }

    /// Removes all `Node`s from the `Octree`.
    pub fn clear(&mut self) {
        self.root = Box::new(Node::<T>::new([
            Vector3::from([0, 0, 0]),
            Vector3::from([self.dimension.get(), self.dimension.get(), self.dimension.get()]),
        ]));
    }

    /// Creates a new `Octree` by decreasing the leaf dimension.
    pub fn new_lod(&self) -> Self {
        let mut root = self.root.clone();
        root.lod();

        Self {
            dimension: self.dimension,
            root,
        }
    }

    /// Effectively decreases the leaf dimension of the `Octree`.
    /// Moves the leaf dimension up a level, and all leaves are formed by the most common data of their
    /// original children.
    pub fn lod(&mut self) {
        self.root.lod();
    }

    /// Returns the dimension of the root node.
    pub fn dimension(&self) -> u32 {
        self.root.dimension()
    }

    /// Returns whether the given position exists within the confines of the `Octree`.
    pub fn contains(&self, position: [u32; 3]) -> bool {
        self.root.contains(position.into())
    }
}
