struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
}

@vertex
fn vertex_main(@location(0) position: vec2f, @location(1) color: vec3f) -> VertexOutput {
    return VertexOutput(vec4f(position, 0.0, 1.0), color);
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
