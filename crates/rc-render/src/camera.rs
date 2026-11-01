use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // radians; 0 looks toward -Z, +90deg toward +X
    pub pitch: f32, // radians, clamped to +-89deg
    pub fov_y: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

const MAX_PITCH: f32 = 89.0_f32 * std::f32::consts::PI / 180.0;

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(8.0, 20.0, 40.0),
            yaw: 0.0,
            pitch: -0.4,
            fov_y: 70.0_f32.to_radians(),
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn forward(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        Vec3::new(sy * cp, sp, -cy * cp)
    }

    /// Horizontal right vector.
    pub fn right(&self) -> Vec3 {
        let (sy, cy) = self.yaw.sin_cos();
        Vec3::new(cy, 0.0, sy)
    }

    pub fn rotate(&mut self, d_yaw: f32, d_pitch: f32) {
        self.yaw += d_yaw;
        self.pitch = (self.pitch + d_pitch).clamp(-MAX_PITCH, MAX_PITCH);
    }

    /// Right-handed, wgpu depth range [0, 1].
    pub fn view_proj(&self) -> Mat4 {
        let proj = Mat4::perspective_rh(self.fov_y, self.aspect, self.near, self.far);
        let view = Mat4::look_to_rh(self.position, self.forward(), Vec3::Y);
        proj * view
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ndc_z(cam: &Camera, dist: f32) -> f32 {
        let p = cam.position + cam.forward() * dist;
        let c = cam.view_proj() * p.extend(1.0);
        c.z / c.w
    }

    #[test]
    fn basis_vectors() {
        let mut c = Camera::new(1.0);
        c.yaw = 0.0;
        c.pitch = 0.0;
        assert!((c.forward() - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-6);
        assert!((c.right() - Vec3::X).length() < 1e-6);
        c.yaw = 90.0_f32.to_radians();
        assert!((c.forward() - Vec3::X).length() < 1e-5);
        assert!((c.right() - Vec3::Z).length() < 1e-5);
    }

    #[test]
    fn depth_range_is_zero_to_one() {
        let c = Camera::new(16.0 / 9.0);
        assert!(ndc_z(&c, c.near).abs() < 1e-3);
        assert!((ndc_z(&c, c.far) - 1.0).abs() < 1e-3);
        let mid = ndc_z(&c, 10.0);
        assert!(mid > 0.0 && mid < 1.0);
        assert!(ndc_z(&c, 5.0) < ndc_z(&c, 50.0)); // monotonic
    }

    #[test]
    fn point_behind_camera_has_negative_w() {
        let c = Camera::new(1.0);
        let p = c.position - c.forward() * 5.0;
        assert!((c.view_proj() * p.extend(1.0)).w < 0.0);
    }

    #[test]
    fn pitch_is_clamped() {
        let mut c = Camera::new(1.0);
        c.rotate(0.0, 10.0);
        assert!(c.pitch <= MAX_PITCH + 1e-6);
        c.rotate(0.0, -20.0);
        assert!(c.pitch >= -MAX_PITCH - 1e-6);
    }
}
