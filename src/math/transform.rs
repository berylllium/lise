use super::{mat4x4::Mat4x4, vec3::Vec3F};

#[derive(Clone, Copy, Default)]
pub struct Transform {
    pub translation: Vec3F,
    pub rotation: Vec3F,
    pub scale: Vec3F,
}

impl Transform {
    pub fn new(translation: Vec3F, rotation: Vec3F, scale: Vec3F) -> Self {
        Self { translation, rotation, scale }
    }

    pub fn new_empty() -> Self {
        Self {
            translation: Vec3F::new_empty(),
            rotation: Vec3F::new_empty(),
            scale: Vec3F::new_empty(),
        }
    }
}

impl Transform {
    pub fn calculate_transformation_matrix(&self) -> Mat4x4 {
        let mut out = Mat4x4::IDENTITY;

        out = out * Mat4x4::from_scale(self.scale);
        out = out * Mat4x4::from_rotation_euler_xyz(self.rotation);
        out = out * Mat4x4::from_translation(self.translation);

        out
    }
}


