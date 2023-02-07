@group(0) @binding(0) var atlas_texture: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler; 

struct Glyph {
    @builtin(vertex_index) index: u32,
    
    @location(0) pos: vec3<f32>,
    @location(1) codepoint: u32,
    @location(2) scale: vec2<f32>,
    @location(3) color: vec4<f32>
}

struct Fragment {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>
}

fn generate_quad(index: u32) -> vec2<f32> {
    switch(index) {
        // Top-right triangle
        case 0u { return vec2<f32>(0.0,  0.0); }
        case 1u { return vec2<f32>(1.0,  0.0); }
        case 2u { return vec2<f32>(1.0, -1.0); }

        // Bottom-left triangle
        case 3u { return vec2<f32>(0.0,  0.0); }
        case 4u { return vec2<f32>(0.0, -1.0); }
        default { return vec2<f32>(1.0, -1.0); }
    }
}
fn glyph_uv(pos: vec2<f32>, codepoint: u32) -> vec2<f32> {
    let glyph_x = f32(codepoint % 16u);
    let glyph_y = f32(codepoint / 16u);

    return vec2<f32>(
        (glyph_x + pos.x) / 16.0,
        (glyph_y - pos.y) / 8.0
    );
}

@vertex
fn vert_main(in: Glyph) -> Fragment {
    var out: Fragment;
    let mesh = generate_quad(in.index);

    out.pos = vec4<f32>(mesh * in.scale + in.pos.xy, in.pos.z, 1.0);
    out.uv = glyph_uv(mesh, in.codepoint);
    out.color = in.color;

    return out;
}


@fragment 
fn frag_main(in: Fragment) -> @location(0) vec4<f32> {
    return in.color * textureSample(atlas_texture, atlas_sampler, in.uv.xy);
}