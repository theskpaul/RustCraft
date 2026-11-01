use rc_render::camera::Camera;

#[derive(Default)]
pub struct FlyInput {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub boost: bool,
    /// Accumulated mouse delta in pixels since last update (taken and reset by `update`).
    pub look: (f32, f32),
}

const SPEED: f32 = 10.0; // blocks/s, tuning value
const BOOST: f32 = 4.0;
const SENSITIVITY: f32 = 0.0025; // rad/pixel

pub fn update(cam: &mut Camera, input: &mut FlyInput, dt: f32) {
    cam.rotate(input.look.0 * SENSITIVITY, -input.look.1 * SENSITIVITY);
    input.look = (0.0, 0.0);

    let axis = |pos: bool, neg: bool| (pos as i32 - neg as i32) as f32;
    let mut v = cam.forward() * axis(input.forward, input.back)
        + cam.right() * axis(input.right, input.left)
        + glam::Vec3::Y * axis(input.up, input.down);
    if v.length_squared() > 0.0 {
        v = v.normalize();
    }
    let speed = SPEED * if input.boost { BOOST } else { 1.0 };
    cam.position += v * speed * dt;
}
