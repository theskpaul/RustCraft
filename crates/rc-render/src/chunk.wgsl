struct Globals {
    view_proj: mat4x4<f32>,
    origin: vec4<f32>,
};
@group(0) @binding(0) var<uniform> g: Globals;

struct VsIn {
    @location(0) a: vec4<u32>, // pos.xyz, face
    @location(1) b: vec2<u32>, // block, corner
};
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec3<f32>,
};

var<private> FACE_SHADE: array<f32, 6> = array<f32, 6>(0.8, 0.8, 1.0, 0.5, 0.9, 0.9);

// Linear-light values. Authored sRGB: magenta (1,0,1), stone grey 0.5,
// dirt (0.45,0.3,0.15), grass (0.3,0.65,0.2). Surface is sRGB, so the
// hardware encodes our output; we must output linear.
var<private> PALETTE: array<vec3<f32>, 4> = array<vec3<f32>, 4>(
    vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(0.214, 0.214, 0.214),
    vec3<f32>(0.1706, 0.0732, 0.0196),
    vec3<f32>(0.0732, 0.3801, 0.0331),
);

@vertex
fn vs_main(in: VsIn) -> VsOut {
    var out: VsOut;
    let local = vec3<f32>(f32(in.a.x), f32(in.a.y), f32(in.a.z));
    out.pos = g.view_proj * vec4<f32>(local + g.origin.xyz, 1.0);
    out.color = PALETTE[min(in.b.x, 3u)] * FACE_SHADE[min(in.a.w, 5u)];
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
