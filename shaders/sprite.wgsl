struct Sprite {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec3<f32>,
    @location(1) scale: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv_a: vec2<f32>,
    @location(4) uv_b: vec2<f32>,
}

struct Fragment {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>
}

fn generate_quad(index: u32) -> vec2<f32> {
    switch(index) {
        // Bottom-right triangle
        case 0u { return vec2<f32>( 1.0,  1.0); }
        case 1u { return vec2<f32>(-1.0, -1.0); }
        case 2u { return vec2<f32>( 1.0, -1.0); }

        // Top-left triangle
        case 3u { return vec2<f32>( 1.0,  1.0); }
        case 4u { return vec2<f32>(-1.0,  1.0); }
        default { return vec2<f32>(-1.0, -1.0); }
    }
}


@vertex
fn vert_main(in: Sprite) -> Fragment {
    var out: Fragment;
    
    return out;
}

@fragment 
fn frag_main(in: Fragment) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
} 
