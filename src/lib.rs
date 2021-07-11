#![allow(dead_code)]

mod node;
mod octree;

pub use octree::Octree;

pub(crate) use node::Node;

#[cfg(test)]
mod tests {
    use super::*;

    use nalgebra::vector;

    use std::num::NonZeroU32;

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
        assert!(matches!(octree.get(vector![20, 1, 12]), None));
    }

    #[test]
    fn contains() {
        let octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();

        assert!(octree.contains(vector![16, 29, 7]));
        assert!(!octree.contains(vector![16, 29, 33]));
    }
}
