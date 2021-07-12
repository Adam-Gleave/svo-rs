use nalgebra::Vector3;

use core::{fmt, num::NonZeroU32};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidDimension(NonZeroU32),
    InvalidPosition(Vector3<u32>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimension(dimension) => write!(f, "Invalid dimension: {}. Must be a power of 2.", dimension),
            Self::InvalidPosition(position) => write!(f, "Position {:?} does not exist in octree.", position),
        }
    }
}
