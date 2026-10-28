#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var source_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var source_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> params: vec4<f32>;

fn sample_source(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(source_texture, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let texel = 1.0 / vec2<f32>(textureDimensions(source_texture));
    let base = sample_source(uv);

    var color = base;
    if params.z > 0.5 {
        if params.x > 0.0 {
            // Eight fixed taps keep the bloom pass bounded on both native and WebGPU.
            let tap_nw = sample_source(uv + texel * vec2<f32>(-1.0, -1.0));
            let tap_n  = sample_source(uv + texel * vec2<f32>( 0.0, -1.0));
            let tap_ne = sample_source(uv + texel * vec2<f32>( 1.0, -1.0));
            let tap_w  = sample_source(uv + texel * vec2<f32>(-1.0,  0.0));
            let tap_e  = sample_source(uv + texel * vec2<f32>( 1.0,  0.0));
            let tap_sw = sample_source(uv + texel * vec2<f32>(-1.0,  1.0));
            let tap_s  = sample_source(uv + texel * vec2<f32>( 0.0,  1.0));
            let tap_se = sample_source(uv + texel * vec2<f32>( 1.0,  1.0));
            let neighborhood = (tap_nw + tap_n + tap_ne + tap_w + tap_e + tap_sw + tap_s + tap_se) * 0.125;
            let glow = max(neighborhood - vec3<f32>(0.52), vec3<f32>(0.0));
            color += glow * clamp(params.x, 0.0, 1.0);
        }

        let row = floor(mesh.position.y);
        let scan = 1.0 - clamp(params.y, 0.0, 0.8) * (0.5 + 0.5 * cos(row * 3.14159265));
        let centered = (uv - vec2<f32>(0.5)) * vec2<f32>(1.0, 1.0);
        let vignette = 1.0 - 0.12 * dot(centered, centered);
        color *= scan * max(vignette, 0.75);
    }

    return vec4<f32>(color, 1.0);
}
