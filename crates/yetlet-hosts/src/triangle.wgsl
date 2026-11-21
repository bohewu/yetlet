// Original neutral triangle; no game scene, domain state or mutable inputs.
@vertex
fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-0.65, -0.55),
        vec2<f32>(0.65, -0.55),
        vec2<f32>(0.0, 0.65),
    );
    return vec4<f32>(positions[index], 0.0, 1.0);
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.75, 0.7, 1.0);
}
