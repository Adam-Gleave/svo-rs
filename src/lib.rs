#![no_std]
#![allow(dead_code)]

extern crate alloc;

#[cfg(any(test, feature = "std"))]
#[macro_use]
extern crate std;

mod error;
mod node;
mod octree;
mod vector;

pub use error::Error;
pub use octree::Octree;

pub(crate) use node::Node;
pub(crate) use vector::Vector3;

#[cfg(test)]
mod tests {
    use bendy::{decoding::FromBencode, encoding::ToBencode};

    use super::*;

    use core::num::NonZeroU32;

    #[test]
    fn simplify_and_insert() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
        octree.insert([0, 0, 0], 1).unwrap();
        octree.insert([0, 0, 1], 1).unwrap();
        octree.insert([0, 1, 0], 1).unwrap();
        octree.insert([0, 1, 1], 1).unwrap();
        octree.insert([1, 0, 0], 1).unwrap();
        octree.insert([1, 0, 1], 1).unwrap();
        octree.insert([1, 1, 0], 1).unwrap();
        octree.insert([1, 1, 1], 1).unwrap();
        octree.insert([0, 0, 0], 2).unwrap();

        assert!(matches!(octree.get([0, 0, 0]), Some(2)));
        assert!(matches!(octree.get([0, 0, 1]), Some(1)));
    }

    #[test]
    fn clear_at_simplified() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
        octree.insert([0, 0, 0], 1).unwrap();
        octree.insert([0, 0, 1], 1).unwrap();
        octree.insert([0, 1, 0], 1).unwrap();
        octree.insert([0, 1, 1], 1).unwrap();
        octree.insert([1, 0, 0], 1).unwrap();
        octree.insert([1, 0, 1], 1).unwrap();
        octree.insert([1, 1, 0], 1).unwrap();
        octree.insert([1, 1, 1], 1).unwrap();

        octree.clear_at([1, 1, 1]).unwrap();

        assert!(matches!(octree.get([1, 1, 1]), Some(0)));
        assert!(matches!(octree.get([0, 0, 0]), Some(1)));
    }

    #[test]
    fn indexed_access() {
        const DIM: u32 = 32;
        let mut octree = Octree::<u32>::new(NonZeroU32::new(DIM as u32).unwrap()).unwrap();
        for x in 0..DIM {
            for y in 0..DIM {
                for z in 0..DIM {
                    let result = octree.insert([x as u32, y as u32, z as u32], (x + y + z + 1).into());
                    assert!(result.is_ok());
                }
            }
        }

        for x in 0..DIM {
            for y in 0..DIM {
                for z in 0..DIM {
                    let expected = x + y + z + 1;
                    let result = octree.get([x as u32, y as u32, z as u32]);
                    assert!(result.is_some());
                    if let Some(value) = result {
                        assert!(*value == expected);
                    };
                }
            }
        }
    }

    #[test]
    fn basic_serialization() {
        let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
        octree.insert([0, 0, 0], 1).unwrap();
        octree.insert([0, 0, 1], 1).unwrap();
        octree.insert([0, 1, 0], 1).unwrap();
        octree.insert([0, 1, 1], 1).unwrap();
        octree.insert([1, 0, 0], 1).unwrap();
        octree.insert([1, 0, 1], 1).unwrap();
        octree.insert([1, 1, 0], 1).unwrap();
        octree.insert([1, 1, 1], 1).unwrap();

        octree.clear_at([1, 1, 1]).unwrap();

        assert!(matches!(octree.get([1, 1, 1]), Some(0)));
        assert!(matches!(octree.get([0, 0, 0]), Some(1)));

        let serialized = octree.to_bencode().unwrap();
        let octree: octree::Octree<u8> = Octree::from_bencode(&serialized).unwrap();
        assert!(matches!(octree.get([1, 1, 1]), Some(0)));
        assert!(matches!(octree.get([0, 0, 0]), Some(1)));
    }

    // #[test]
    // fn test() {
    //     let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();
        
    //     octree.insert([0, 0, 0], 1).unwrap();
    //     octree.insert([0, 0, 1], 1).unwrap();
    //     octree.insert([0, 1, 0], 1).unwrap();
    //     octree.insert([0, 1, 1], 1).unwrap();
    //     octree.insert([1, 0, 0], 1).unwrap();
    //     octree.insert([1, 0, 1], 1).unwrap();
    //     octree.insert([1, 1, 0], 1).unwrap();
    //     octree.insert([1, 1, 1], 1).unwrap();

        // octree.insert([0, 0, 2], 2).unwrap();
        // octree.insert([1, 0, 2], 2).unwrap();
        // octree.insert([0, 0, 3], 2).unwrap();
        // octree.insert([1, 0, 3], 2).unwrap();
        // octree.insert([0, 1, 2], 2).unwrap();
        // octree.insert([1, 1, 2], 2).unwrap();
        // octree.insert([0, 1, 3], 2).unwrap();
        // octree.insert([1, 1, 3], 2).unwrap();

        // octree.insert([0, 2, 0], 3).unwrap();
        // octree.insert([1, 2, 0], 3).unwrap();
        // octree.insert([0, 2, 1], 3).unwrap();
        // octree.insert([1, 2, 1], 3).unwrap();
        // octree.insert([0, 3, 0], 3).unwrap();
        // octree.insert([1, 3, 0], 3).unwrap();
        // octree.insert([0, 3, 1], 3).unwrap();
        // octree.insert([1, 3, 1], 3).unwrap();

        // octree.insert([2, 0, 0], 4).unwrap();
        // octree.insert([3, 0, 0], 4).unwrap();
        // octree.insert([2, 0, 1], 4).unwrap();
        // octree.insert([3, 0, 1], 4).unwrap();
        // octree.insert([2, 1, 0], 4).unwrap();
        // octree.insert([3, 1, 0], 4).unwrap();
        // octree.insert([2, 1, 1], 4).unwrap();
        // octree.insert([3, 1, 1], 4).unwrap();

        // println!("{:?}", octree);
    // }
}
