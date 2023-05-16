use crate::{Error, Node, Vector3};

#[cfg(feature = "no-std")]
use micromath::F32Ext;

use alloc::boxed::Box;
use core::{f32, hash::Hash, num::NonZeroU32};

pub struct Octree<T>
where
    T: Default + Clone + Eq + PartialEq + Copy + Hash + ToBencode + FromBencode,
{
    pub auto_simplify: bool,
    dimension: NonZeroU32,
    curr_lod_level: u32,
    max_lod_level: u32,
    min_dimension: u32,
    root: Box<Node<T>>,
}

use std::vec::Vec;
impl<T> Octree<T>
where
    T: Default + Clone + Eq + PartialEq + Copy + Hash + ToBencode + FromBencode,
{
    /// Creates a new `Octree<T>` of given dimension.
    ///
    /// Valid dimensions are:
    /// * 1 (a single node, although this is pretty much useless)
    /// * *n*, where *n* is a square number (the `Octree` will consist of n*n nodes)
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap());
    /// assert!(octree.is_ok());
    ///
    /// let octree = Octree::<u8>::new(NonZeroU32::new(15).unwrap());
    /// assert!(matches!(octree, Err(Error::InvalidDimension(15))));
    /// ```
    pub fn new(dimension: NonZeroU32) -> Result<Self, Error> {
        // Check that `dimension` is a power of 2.
        let max_depth = (dimension.get() as f32).log(2.0);

        if max_depth.fract() == 0.0 {
            Ok(Self {
                dimension,
                curr_lod_level: 1,
                max_lod_level: max_depth.round() as u32,
                min_dimension: 1,
                auto_simplify: false,
                root: Box::new(Node::<T>::new(Vector3::from([0, 0, 0]), dimension.get())),
            })
        } else {
            Err(Error::InvalidDimension(dimension.into()))
        }
    }

    /// Inserts data of type `T` into the given position in the `Octree`.
    /// Returns an error if the position does not exist within the confines of the `Octree`.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    /// let res = octree.insert([9, 8, 31], 1);
    ///
    /// assert!(res.is_ok());
    /// ```
    pub fn insert(&mut self, position: [u32; 3], data: T) -> Result<(), Error> {
        self.root.insert(position.into(), self.min_dimension, self.auto_simplify, data)
    }

    /// Retrieves data of type `T` from the given position in the `Octree`.
    /// Since the `Octree` is sparse, returns `None` if the position does not currently store any data.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    /// octree.insert([9, 8, 31], 1).unwrap();
    ///
    /// assert!(matches!(octree.get([9, 8, 31]), Some(1)));
    /// assert!(octree.get([20, 1, 12]).is_none());
    /// ```
    pub fn get(&self, position: [u32; 3]) -> Option<&T> {
        self.root.get(position.into())
    }

    /// Removes the `Node` at the given position in the `Octree`, if it exists.
    /// This will simplify the `Octree` if `auto_simplify` is specified.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    ///
    /// octree.insert([0, 0, 0], 1).unwrap();
    /// octree.insert([0, 0, 1], 1).unwrap();
    /// octree.clear_at([0, 0, 0]).unwrap();
    /// octree.clear_at([0, 0, 1]).unwrap();
    ///
    /// assert!(matches!(octree.get([0, 0, 0]), Some(0)));
    /// assert!(matches!(octree.get([0, 0, 1]), Some(0)));
    ///
    /// octree.insert([31, 31, 31], 1).unwrap();
    /// octree.insert([0, 0, 0], 1).unwrap();
    ///
    /// assert!(matches!(octree.get([31, 31, 31]), Some(1)));
    /// assert!(matches!(octree.get([0, 0, 0]), Some(1)));
    /// ```
    pub fn clear_at(&mut self, position: [u32; 3]) -> Result<(), Error> {
        self.root.clear(position.into(), self.min_dimension)
    }

    /// Removes all `Node`s from the `Octree`.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    ///
    /// octree.insert([0, 0, 0], 1).unwrap();
    /// octree.insert([0, 0, 1], 1).unwrap();
    ///
    /// octree.clear();
    ///
    /// assert!(matches!(octree.get([0, 0, 0]), Some(0)));
    /// assert!(matches!(octree.get([0, 0, 1]), Some(0)));
    /// ```
    pub fn clear(&mut self) {
        self.root = Box::new(Node::<T>::new(Vector3::from([0, 0, 0]), self.dimension.into()));
    }

    /// Effectively increases the leaf dimension of the `Octree` and simplifies where possible.
    ///
    /// Moves the leaf dimension down a level, and all leaves are formed by the most common data of their
    /// original children.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    /// octree.insert([0, 0, 0], 2).unwrap();
    /// octree.insert([0, 0, 1], 2).unwrap();
    /// octree.insert([0, 1, 0], 1).unwrap();
    /// octree.insert([0, 1, 1], 2).unwrap();
    /// octree.insert([1, 0, 0], 1).unwrap();
    /// octree.insert([1, 0, 1], 2).unwrap();
    /// octree.insert([1, 1, 0], 2).unwrap();
    /// octree.insert([1, 1, 1], 1).unwrap();
    ///
    /// octree.lod_down();
    /// assert!(matches!(octree.get([0, 1, 0]), Some(2)));
    /// ```
    pub fn lod_down(&mut self) {
        let level = if self.curr_lod_level + 1 >= self.max_lod_level {
            self.max_lod_level
        } else {
            self.curr_lod_level + 1
        };

        let min_dimension = 2_u32.pow(level - 1);

        self.root.lod();
        self.curr_lod_level = level;
        self.min_dimension = min_dimension;
    }

    /// Effectively decreases the leaf dimension of the `Octree`.
    ///
    /// Note that the structure of the `Octree` does not change, as it cannot "remember" old, higher LOD
    /// levels. Rather, this method allows the insertion of new leaf nodes at a higher detail level.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    /// octree.insert([0, 0, 0], 2).unwrap();
    /// octree.insert([0, 0, 1], 2).unwrap();
    /// octree.insert([0, 1, 0], 1).unwrap();
    /// octree.insert([0, 1, 1], 2).unwrap();
    /// octree.insert([1, 0, 0], 1).unwrap();
    /// octree.insert([1, 0, 1], 2).unwrap();
    /// octree.insert([1, 1, 0], 2).unwrap();
    /// octree.insert([1, 1, 1], 1).unwrap();
    ///
    /// octree.lod_down();
    /// assert!(matches!(octree.get([0, 1, 0]), Some(2)));
    ///
    /// octree.lod_up();
    /// octree.insert([0, 0, 0], 1).unwrap();
    /// assert!(matches!(octree.get([0, 0, 0]), Some(1)));
    /// assert!(matches!(octree.get([0, 0, 1]), Some(2)));
    /// ```
    pub fn lod_up(&mut self) {
        let level = if self.curr_lod_level - 1 <= 0 {
            1
        } else {
            self.curr_lod_level - 1
        };

        let min_dimension = 2_u32.pow(level - 1);

        self.curr_lod_level = level;
        self.min_dimension = min_dimension;
    }

    /// Returns the dimension of the root node.
    pub fn dimension(&self) -> u32 {
        self.root.dimension()
    }

    /// Returns whether the given position exists within the confines of the `Octree`.
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// let octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    ///
    /// assert!(octree.contains([16, 29, 7]));
    /// assert!(!octree.contains([16, 29, 33]));
    /// ```
    pub fn contains(&self, position: [u32; 3]) -> bool {
        self.root.contains(position.into())
    }

    /// Simplifies the nodes wherever possible
    ///
    /// Returns with wether or not the root Node could be simplified
    ///
    /// # Example
    /// ```
    /// # use svo_rs::{Error, Octree};
    /// # use core::num::NonZeroU32;
    /// #
    /// const DIM: u32 = 32;
    /// let mut octree = Octree::<u32>::new(NonZeroU32::new(DIM as u32).unwrap()).unwrap();
    /// let data_field = |x: u32, y: u32, z: u32| -> u32 {
    ///     5//(((x as f32).sin() + (y as f32).sin() + (z as f32).sin()).min(-1.) * -100.) as u32
    /// };
    /// for x in 0..DIM {
    ///     for y in 0..DIM {
    ///         for z in 0..DIM {
    ///             let result = octree.insert([x as u32, y as u32, z as u32], data_field(x, y, z));
    ///             assert!(result.is_ok());
    ///         }
    ///     }
    /// }
    /// assert!(octree.simplify());
    /// for x in 0..DIM {
    ///     for y in 0..DIM {
    ///         for z in 0..DIM {
    ///             let result = octree.get([x as u32, y as u32, z as u32]);
    ///             assert!(result.is_some());
    ///             if let Some(value) = result {
    ///                 assert_eq!(*value, data_field(x, y, z));
    ///             };
    ///         }
    ///     }
    /// }
    /// ``` 
    pub fn simplify(&mut self) -> bool{
        self.root.simplify_recursive()
    }

    pub fn serialize(&self)-> Vec<(&Node<T>, [usize; crate::node::OCTREE_CHILDREN])>{
        self.root.serialize()
    }

    pub fn deserialize(&mut self, all_nodes: Vec<(Option<Node<T>>, [usize; crate::node::OCTREE_CHILDREN])>){
        self.root = Box::new(Node::<T>::deserialize(all_nodes));
    }

}

use bendy::encoding::{SingleItemEncoder, ToBencode};
impl<T> ToBencode for Octree<T>
where
    T: Default + Clone + Eq + PartialEq + Copy + Hash + ToBencode + FromBencode,
{
    const MAX_DEPTH: usize = 5; //TODO: does this need to include depth of the Node trait implementation?
    fn encode(&self, encoder: SingleItemEncoder) -> Result<(), bendy::encoding::Error> {
        encoder.emit_list(|e| {
            e.emit_int(u32::from(self.dimension))?;
            e.emit_int(self.curr_lod_level)?;
            e.emit_int(self.max_lod_level)?;
            e.emit_int(self.min_dimension)?;
            e.emit_int(self.auto_simplify as i8)?;
            e.emit(self.root.clone()) //TODO: Does this really need to be cloned?
        })
    }
}
use bendy::decoding::{FromBencode, Object};

impl<T> FromBencode for Octree<T>
where
    T: Default + Clone + Eq + PartialEq + Copy + Hash + ToBencode + FromBencode,
{
    fn decode_bencode_object(data: Object) -> Result<Self, bendy::decoding::Error> {
        match data {
            Object::List(mut list) => {
                let dimension = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse::<NonZeroU32>().unwrap()),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "Integer Octree dimension",
                        "Something else",
                    )),
                }?;

                let curr_lod_level = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "Integer Octree curr_lod_level",
                        "Something else",
                    )),
                }?;

                let max_lod_level = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "Integer Octree max_lod_level",
                        "Something else",
                    )),
                }?;

                let min_dimension = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse::<u32>().unwrap()),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "Integer Octree min_dimension",
                        "Something else",
                    )),
                }?;

                let auto_simplify = match list.next_object()?.unwrap() {
                    Object::Integer(i) => Ok(i.parse::<u8>().unwrap()),
                    _ => Err(bendy::decoding::Error::unexpected_token(
                        "Boolean for octree Auto simplify",
                        "Something else",
                    )),
                }?;

                let root = Node::<T>::decode_bencode_object(list.next_object()?.unwrap())?;
                Ok(Octree {
                    dimension,
                    curr_lod_level,
                    max_lod_level,
                    min_dimension,
                    auto_simplify: 0 < auto_simplify,
                    root: Box::new(root),
                })
            }
            _ => Err(bendy::decoding::Error::unexpected_token("List", "not List")),
        }
    }
}
