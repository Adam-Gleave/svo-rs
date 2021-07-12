#![no_std]
#![allow(dead_code)]

extern crate alloc;

#[cfg(test)]
#[macro_use]
extern crate std;

mod error;
mod node;
mod octree;

pub use error::Error;
pub use octree::Octree;

pub(crate) use node::Node;

#[cfg(test)]
mod tests {
    use super::*;

    use nalgebra::vector;

    use core::num::NonZeroU32;

    #[test]
    fn new_valid() {
        let octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap());
        assert!(octree.is_ok());
    }

    #[test]
    fn insert() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
        let res = octree.insert(vector![9, 8, 31], 1);

        assert!(res.is_ok());
    }

    #[test]
    fn get() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
        octree.insert(vector![9, 8, 31], 1).unwrap();

        assert!(matches!(octree.get(vector![9, 8, 31]), Some(1)));
        assert!(octree.get(vector![20, 1, 12]).is_none());
    }

    #[test]
    fn clear() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap())
            .unwrap();

        octree.insert(vector![0, 0, 0], 1).unwrap();
        octree.insert(vector![0, 0, 1], 1).unwrap();

        octree.clear();

        assert!(octree.get(vector![0, 0, 0]).is_none());
        assert!(octree.get(vector![0, 0, 1]).is_none());
    }

    #[test]
    fn clear_at() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap())
            .unwrap();

        octree.insert(vector![0, 0, 0], 1).unwrap();
        octree.insert(vector![0, 0, 1], 1).unwrap();
        octree.clear_at(vector![0, 0, 0]).unwrap();
        octree.clear_at(vector![0, 0, 1]).unwrap();

        assert!(octree.get(vector![0, 0, 0]).is_none());
        assert!(octree.get(vector![0, 0, 1]).is_none());

        octree.insert(vector![31, 31, 31], 1).unwrap();
        octree.insert(vector![0, 0, 0], 1).unwrap();

        assert!(matches!(octree.get(vector![31, 31, 31]), Some(1)));
        assert!(matches!(octree.get(vector![0, 0, 0]), Some(1)));
    }

    // #[test]
    // fn clear_at_simplified() {
    //     let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
    //     octree.insert(vector![0, 0, 0], 1).unwrap();
    //     octree.insert(vector![0, 0, 1], 1).unwrap();
    //     octree.insert(vector![0, 1, 0], 1).unwrap();
    //     octree.insert(vector![0, 1, 1], 1).unwrap();
    //     octree.insert(vector![1, 0, 0], 1).unwrap();
    //     octree.insert(vector![1, 0, 1], 1).unwrap();
    //     octree.insert(vector![1, 1, 0], 1).unwrap();
    //     octree.insert(vector![1, 1, 1], 1).unwrap();

    //     octree.clear_at(vector![1, 1, 1]).unwrap();

    //     println!("{:?}", octree);
    // }

    #[test]
    fn contains() {
        let octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();

        assert!(octree.contains(vector![16, 29, 7]));
        assert!(!octree.contains(vector![16, 29, 33]));
    }
}
