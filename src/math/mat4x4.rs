use super::vec3::Vec3F;

#[derive(Copy, Clone)]
pub struct Mat4x4 {
    pub data: [f32; 16],
}

impl std::ops::Mul for Mat4x4 {
    type Output = Self;
    
    fn mul(self, rhs: Self) -> Self {
        let mut out = Self::IDENTITY;

        let mut l_ptr = self.data.as_ptr();
        let r_ptr = rhs.data.as_ptr();
        let mut dst_ptr = out.data.as_mut_ptr();

        for _ in 0..4 {
            for i in 0..4 {
                unsafe {
                    *dst_ptr = 
                        *l_ptr.add(0) * *r_ptr.add(i) +
                        *l_ptr.add(1) * *r_ptr.add(4 + i) +
                        *l_ptr.add(2) * *r_ptr.add(8 + i) +
                        *l_ptr.add(3) * *r_ptr.add(12 + i);

                    dst_ptr = dst_ptr.add(1);
                }
                l_ptr = unsafe { l_ptr.add(4) };
            }
        }
        
        out
    }
}

impl Mat4x4 {
    pub const ZERO: Mat4x4 = Mat4x4 {
        data: [ 
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        ],
    };

    pub const IDENTITY: Mat4x4 = Mat4x4 {
        data: [ 
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
    };

    pub fn new_perspective(fov_rads: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let half_tan_fov = (fov_rads * 0.5).tan();

        let mut out = Mat4x4::IDENTITY;
        
    	out.data[0] = 1.0 / (aspect_ratio * half_tan_fov);
	    out.data[5] = 1.0 / half_tan_fov;
	    out.data[10] = -((far + near) / (far - near));
	    out.data[11] = -1.0;
	    out.data[14] = -((2.0 * far * near) / (far - near));

        out
    }

    pub fn new_look_at(position: Vec3F, target: Vec3F, up: Vec3F) -> Self {
        let mut out = Mat4x4::ZERO;
	    let z_axis = Vec3F::new(
            target.x - position.x,
            target.y - position.y,
            target.z - position.z,
        ).to_normalized();

	    let x_axis = z_axis.cross(up).to_normalized();
	    let y_axis = x_axis.cross(z_axis);

	    out.data[0] = x_axis.x;
	    out.data[1] = y_axis.x;
	    out.data[2] = -z_axis.x;
	    out.data[3] = 0.0;
	    out.data[4] = x_axis.y;
	    out.data[5] = y_axis.y;
	    out.data[6] = -z_axis.y;
	    out.data[7] = 0.0;
	    out.data[8] = x_axis.z;
	    out.data[9] = y_axis.z;
	    out.data[10] = -z_axis.z;
	    out.data[11] = 0.0;
	    out.data[12] = -x_axis.dot(position);
	    out.data[13] = -y_axis.dot(position);
	    out.data[14] = z_axis.dot(position);
	    out.data[15] = 1.0;

        out
    }

    pub fn from_translation(translation: Vec3F) -> Self {
        let mut out = Self::IDENTITY;

        out.data[12] = translation.x;
        out.data[13] = translation.y;
        out.data[14] = translation.z;

        out
    }

    pub fn from_scale(scale: Vec3F) -> Self {
        let mut out = Self::IDENTITY;

        out.data[0] = scale.x;
        out.data[5] = scale.y;
        out.data[10] = scale.z;

        out
    }

    pub fn from_rotation_euler_x(rad: f32) -> Self {
        let c = rad.cos();
        let s = rad.sin();

        let mut out = Self::IDENTITY;

        out.data[5] = c;
        out.data[6] = s;
        out.data[9] = -s;
        out.data[10] = c;
        
        out
    }

    pub fn from_rotation_euler_y(rad: f32) -> Self {
        let c = rad.cos();
        let s = rad.sin();

        let mut out = Self::IDENTITY;

        out.data[0] = c;
        out.data[2] = -s;
        out.data[8] = s;
        out.data[10] = c;
        
        out
    }

    pub fn from_rotation_euler_z(rad: f32) -> Self {
        let c = rad.cos();
        let s = rad.sin();

        let mut out = Self::IDENTITY;

        out.data[0] = c;
        out.data[1] = s;
        out.data[4] = -s;
        out.data[5] = c;
        
        out
    }

    pub fn from_rotation_euler_xyz(rotation: Vec3F) -> Self {
        let rx = Self::from_rotation_euler_x(rotation.x);
        let ry = Self::from_rotation_euler_y(rotation.y);
        let rz = Self::from_rotation_euler_z(rotation.z);

        rx * ry * rz
    }
}

impl Mat4x4 {
    pub fn to_transposed(&self) -> Self {
        let mut out = Self::IDENTITY;

        out.data[0] = self.data[0];
        out.data[1] = self.data[4];
        out.data[2] = self.data[8];
        out.data[3] = self.data[12];
        out.data[4] = self.data[1];
        out.data[5] = self.data[5];
        out.data[6] = self.data[9];
        out.data[7] = self.data[13];
        out.data[8] = self.data[2];
        out.data[9] = self.data[6];
        out.data[10] = self.data[10];
        out.data[11] = self.data[14];
        out.data[12] = self.data[3];
        out.data[13] = self.data[7];
        out.data[14] = self.data[11];
        out.data[15] = self.data[15];

        out
    }

    pub fn to_inverted(&self) -> Self {
        let m = &self.data;

        let t0 = m[10] * m[15];
        let t1 = m[14] * m[11];
        let t2 = m[6] * m[15];
        let t3 = m[14] * m[7];
        let t4 = m[6] * m[11];
        let t5 = m[10] * m[7];
        let t6 = m[2] * m[15];
        let t7 = m[14] * m[3];
        let t8 = m[2] * m[11];
        let t9 = m[10] * m[3];
        let t10 = m[2] * m[7];
        let t11 = m[6] * m[3];
        let t12 = m[8] * m[13];
        let t13 = m[12] * m[9];
        let t14 = m[4] * m[13];
        let t15 = m[12] * m[5];
        let t16 = m[4] * m[9];
        let t17 = m[8] * m[5];
        let t18 = m[0] * m[13];
        let t19 = m[12] * m[1];
        let t20 = m[0] * m[9];
        let t21 = m[8] * m[1];
        let t22 = m[0] * m[5];
        let t23 = m[4] * m[1];

        let mut out = Self::ZERO;
        let o = &mut out.data;

        o[0] = (t0 * m[5] + t3 * m[9] + t4 * m[13]) - (t1 * m[5] + t2 * m[9] + t5 * m[13]);
        o[1] = (t1 * m[1] + t6 * m[9] + t9 * m[13]) - (t0 * m[1] + t7 * m[9] + t8 * m[13]);
        o[2] = (t2 * m[1] + t7 * m[5] + t10 * m[13]) - (t3 * m[1] + t6 * m[5] + t11 * m[13]);
        o[3] = (t5 * m[1] + t8 * m[5] + t11 * m[9]) - (t4 * m[1] + t9 * m[5] + t10 * m[9]);

        let d = 1.0f32 / (m[0] * o[0] + m[4] * o[1] + m[8] * o[2] + m[12] * o[3]);

        o[0] *= d;
        o[1] *= d; 
        o[2] *= d; 
        o[3] *= d; 
        o[4] = d * ((t1 * m[4] + t2 * m[8] + t5 * m[12]) - (t0 * m[4] + t3 * m[8] + t4 * m[12]));
        o[5] = d * ((t0 * m[0] + t7 * m[8] + t8 * m[12]) - (t1 * m[0] + t6 * m[8] + t9 * m[12]));
        o[6] = d * ((t3 * m[0] + t6 * m[4] + t11 * m[12]) - (t2 * m[0] + t7 * m[4] + t10 * m[12]));
        o[7] = d * ((t4 * m[0] + t9 * m[4] + t10 * m[8]) - (t5 * m[0] + t8 * m[4] + t11 * m[8]));
        o[8] = d * ((t12 * m[7] + t15 * m[11] + t16 * m[15]) - (t13 * m[7] + t14 * m[11] + t17 * m[15]));
        o[9] = d * ((t13 * m[3] + t18 * m[11] + t21 * m[15]) - (t12 * m[3] + t19 * m[11] + t20 * m[15]));
        o[10] = d * ((t14 * m[3] + t19 * m[7] + t22 * m[15]) - (t15 * m[3] + t18 * m[7] + t23 * m[15]));
        o[11] = d * ((t17 * m[3] + t20 * m[7] + t23 * m[11]) - (t16 * m[3] + t21 * m[7] + t22 * m[11]));
        o[12] = d * ((t14 * m[10] + t17 * m[14] + t13 * m[6]) - (t16 * m[14] + t12 * m[6] + t15 * m[10]));
        o[13] = d * ((t20 * m[14] + t12 * m[2] + t19 * m[10]) - (t18 * m[10] + t21 * m[14] + t13 * m[2]));
        o[14] = d * ((t18 * m[6] + t23 * m[14] + t15 * m[2]) - (t22 * m[14] + t14 * m[2] + t19 * m[6]));
        o[15] = d * ((t22 * m[10] + t16 * m[2] + t21 * m[6]) - (t20 * m[6] + t23 * m[10] + t17 * m[2]));

        out
    }

    pub fn to_forward_vector(&self) -> Vec3F {
        let mut out = Vec3F::new(0.0, 0.0, 0.0);

        out.x = -self.data[2];
        out.y = -self.data[6];
        out.z = -self.data[10];

        out.normalize();

        out
    }

    pub fn to_backward_vector(&self) -> Vec3F {
        let mut out = Vec3F::new(0.0, 0.0, 0.0);

        out.x = self.data[2];
        out.y = self.data[6];
        out.z = self.data[10];

        out.normalize();

        out
    }

    pub fn to_up_vector(&self) -> Vec3F {
        let mut out = Vec3F::new(0.0, 0.0, 0.0);

        out.x = self.data[1];
        out.y = self.data[5];
        out.z = self.data[9];

        out.normalize();

        out
    }

    pub fn to_down_vector(&self) -> Vec3F {
        let mut out = Vec3F::new(0.0, 0.0, 0.0);

        out.x = -self.data[1];
        out.y = -self.data[5];
        out.z = -self.data[9];

        out.normalize();

        out
    }

    pub fn to_left_vector(&self) -> Vec3F {
        let mut out = Vec3F::new(0.0, 0.0, 0.0);

        out.x = -self.data[0];
        out.y = -self.data[4];
        out.z = -self.data[8];

        out.normalize();

        out
    }

    pub fn to_right_vector(&self) -> Vec3F {
        let mut out = Vec3F::new(0.0, 0.0, 0.0);

        out.x = self.data[0];
        out.y = self.data[4];
        out.z = self.data[8];

        out.normalize();

        out
    }
}
