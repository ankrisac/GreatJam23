struct VertexInput {
    @builtin(vertex_index) index: u32,
    @location(0) pos: vec4<f32>,
    @location(1) col: vec4<f32>
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>, 
    @location(0) col: vec4<f32>
}

@vertex
fn vert_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.pos = in.pos;
    out.col = in.col;

    return out;
}

@fragment
fn frag_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.col;
}