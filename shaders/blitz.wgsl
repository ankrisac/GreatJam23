@vertex
fn vert_main(@builtin(vertex_index) in: u32) -> @builtin(position) vec4<f32> {
    var out: vec4<f32> = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    
    switch(in) {
        case 0u { return vec4<f32>( 1.0, -1.0, 0.0, 1.0); }
        case 1u { return vec4<f32>(-1.0,  1.0, 0.0, 1.0); }
        default { return vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
    }
}

@fragment
fn frag_main() {
    
}