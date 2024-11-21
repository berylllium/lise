use std::ops::{Add, Sub};

use ash::vk;
use num_traits::{AsPrimitive, Float, FromPrimitive, Num};

pub type Vec3F = Vec3<f32>;
pub type Vec3I = Vec3<i32>;
pub type Vec3UI = Vec3<u32>;

#[derive(Clone, Copy, Default)]
pub struct Vec3<T: Num> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Num> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn new_empty() -> Self {
        Self { x: T::zero(),  y: T::zero(), z: T::zero() }
    }
}

impl<T: Num + Float> Vec3<T> {
    pub fn length_squared(&self) -> T {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.length();

        self.x = self.x / len;
        self.y = self.y / len;
        self.z = self.z / len;
    }

    pub fn to_normalized(&self) -> Self {
        let mut out = *self;

        out.normalize();

        out
    }

    pub fn dot(&self, rhs: Self) -> T {
        let mut p = T::zero();

        p = p + self.x * rhs.x;
        p = p + self.y * rhs.y;
        p = p + self.z * rhs.z;

        p
    }

    pub fn cross(&self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
			self.z * rhs.x - self.x * rhs.z,
			self.x * rhs.y - self.y * rhs.x,
        )
    }
}

impl<T: Num> Add for Vec3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
        )
    }
}

impl<T: Num> Sub for Vec3<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
        )
    }
}

impl<T: Num> PartialEq for Vec3<T> {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }

    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y || self.z != other.z
    }
}

impl Vec3UI {
    pub fn as_vk_extent(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.x,
            height: self.y,
            depth: self.z,
        }
    }
}
