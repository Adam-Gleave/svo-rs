use core::{fmt, num::NonZeroU32};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidDimension(NonZeroU32),
    InvalidPosition { x: u32, y: u32, z: u32 },
    InvalidOctant(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension(dimension) => write!(f, "Invalid dimension: {}. Must be a power of 2.", dimension),
            Self::InvalidPosition { x, y, z } => {
                write!(f, "Position {{{}, {}, {}}} does not exist in octree.", x, y, z)
            }
            Self::InvalidOctant(octant) => write!(f, "Invalid octant: {}", octant),
        }
    }
}
