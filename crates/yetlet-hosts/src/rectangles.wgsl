struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) index: u32,
    @location(0) center: vec2<f32>,
    @location(1) half_extent: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    let corners = array(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(-1.0, 1.0),
        vec2<f32>(-1.0, 1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
    );
    return VertexOutput(vec4<f32>(center + corners[index] * half_extent, 0.0, 1.0), color);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
