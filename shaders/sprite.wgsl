
struct Instance {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) rot: f32
}

fn generate_mesh(index: u32) -> vec3<f32> {
    switch(index) {
        case 0u { return vec3<f32>( 1.0,  1.0, 0.0); }
        case 1u { return vec3<f32>(-1.0, -1.0, 0.0); }
        case 2u { return vec3<f32>( 1.0, -1.0, 0.0); }
        case 3u { return vec3<f32>( 1.0,  1.0, 0.0); }
        case 4u { return vec3<f32>(-1.0,  1.0, 0.0); }
        default { return vec3<f32>(-1.0, -1.0, 0.0); }
    }
}

@group(0) @binding(0) 
var atlas_texture: texture_2d<f32>;

@group(0) @binding(1) 
var atlas_sampler: sampler;

@vertex 
fn vert_main(in: Instance) -> Fragment {
    let mesh: vec3<f32> = generate_mesh(in.index);

    var out: Fragment;
    let worldpos = in.pos + 0.5 * mesh * mat3x3(
        cos(in.rot), -sin(in.rot), 0.0,
        sin(in.rot),  cos(in.rot), 0.0,
        0.0, 0.0, 1.0,
    );;

    out.pos = vec4<f32>(worldpos * 0.2, 1.0);
    out.uv = vec2<f32>(0.5, 0.5) + 0.5 * mesh.xy;
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return out;
}

struct Fragment {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>
}

@fragment
fn frag_main(in: Fragment) -> @location(0) vec4<f32> {
    let modulation = sin(in.pos.y);
    let tint = vec4<f32>(0.4, 0.95, 0.65, 1.0);

    return tint * modulation * in.color * textureSample(atlas_texture, atlas_sampler, in.uv);
}