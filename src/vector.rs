use core::ops::{Add, Mul};

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Vector3<T>
where
    T: Copy,
{
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Mul<Output = T> + Copy> Vector3<T> {
    pub(crate) fn component_mul(self, other: &Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl<T: Add<Output = T> + Copy> Add for Vector3<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T: Copy> From<[T; 3]> for Vector3<T> {
    fn from(v: [T; 3]) -> Self {
        Self {
            x: v[0],
            y: v[1],
            z: v[2],
        }
    }
}
