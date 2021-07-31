# svo-rs
Sparse Voxel Octree (SVO) library, entirely `#![no_std]`.

# Usage
```rust
use svo_rs::Octree;

// Create an `Octree` with dimensions of 32*32*32, to store `u8` data.
let mut octree = Octree::<u8>::new(NonZeroU32::new(32).unwrap()).unwrap();

// Insert a value of 1 into the `Octree` at position [0, 0, 0].
octree.insert([0, 0, 0], 1);
assert!(matches!(octree.get([0, 0, 0], Some(1))));

// Clear this value from the `Octree`.
octree.clear_at([0, 0, 0]).unwrap();
assert!(octree.get([0, 0, 0]).is_none());

// `Octree` simplification used where possible.
// The following will now be condensed to a single leaf node with dimensions of 2*2*2:
octree.insert([0, 0, 0], 1).unwrap();
octree.insert([0, 0, 1], 1).unwrap();
octree.insert([0, 1, 0], 1).unwrap();
octree.insert([0, 1, 1], 1).unwrap();
octree.insert([1, 0, 0], 1).unwrap();
octree.insert([1, 0, 1], 1).unwrap();
octree.insert([1, 1, 0], 1).unwrap();
octree.insert([1, 1, 1], 1).unwrap();

// Clear the entire `Octree`.
octree.clear();
assert!(octree.get(vector![0, 0, 0]).is_none());
```
