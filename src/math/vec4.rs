use std::ops::{Add, Mul, Sub};

pub type Vec4F = Vec4<f32>;
pub type Vec4I = Vec4<i32>;

pub struct Vec4<T: PartialEq + PartialOrd + Add + Sub + Mul> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: PartialEq + PartialOrd + Add + Sub + Mul> Vec4<T> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }
}
