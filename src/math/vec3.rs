use std::ops::{Add, Mul, Sub};

use ash::vk;

pub type Vec3F = Vec3<f32>;
pub type Vec3I = Vec3<i32>;
pub type Vec3UI = Vec3<u32>;

#[derive(Clone, Copy)]
pub struct Vec3<T: PartialEq + PartialOrd + Add + Sub + Mul + Copy + Clone> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: PartialEq + PartialOrd + Add + Sub + Mul + Copy + Clone> Vec3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
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
