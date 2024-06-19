use std::ops::{Add, Mul, Sub};

use ash::vk;

pub type Vec2F = Vec2<f32>;
pub type Vec2I = Vec2<i32>;
pub type Vec2UI = Vec2<u32>;

#[derive(Clone, Copy, Default)]
pub struct Vec2<T: PartialEq + PartialOrd + Add + Sub + Mul> {
    pub x: T,
    pub y: T,
}

impl<T: PartialEq + PartialOrd + Add + Sub + Mul> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Vec2UI {
    pub fn from_vk_extent_2d(extent: vk::Extent2D) -> Self {
        Self { x: extent.width, y: extent.height }
    }
}

impl Vec2UI {
    pub fn as_vk_extent_2d(&self) -> vk::Extent2D {
        vk::Extent2D {
            width: self.x,
            height: self.y,
        }
    }

    pub fn as_vk_extent_3d(&self, depth: u32) -> vk::Extent3D {
        vk::Extent3D {
            width: self.x,
            height: self.y,
            depth,
        }
    }
}
