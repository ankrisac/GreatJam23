struct Glyph {
    @builtin(vertex_index) index: u32
    
    //@location(0) pos: vec4<f32>,
    //@location(1) scale: vec2<f32>,
    //@location(2) codepoint: u32
}

struct Fragment {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>
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
fn glyph_uv(pos: vec2<f32>) -> vec2<f32> {
    var uv: vec2<f32>;
    uv.x = pos.x;
    uv.y = 1.0 - pos.y;
    return uv;
}

@vertex
fn vert_main(in: Glyph) -> Fragment {
    var out: Fragment;
    let mesh = generate_quad(in.index);

    out.pos = vec4<f32>(mesh, 0.0, 1.0);
    out.uv = glyph_uv(mesh);
    
    return out;
}

@group(0) @binding(0) var atlas_texture: texture_2d<f32>;
@group(0) @binding(1) var atlas_sampler: sampler; 


@fragment 
fn frag_main(in: Fragment) -> @location(0) vec4<f32> {
    return textureSample(atlas_texture, atlas_sampler, in.uv.xy);
}