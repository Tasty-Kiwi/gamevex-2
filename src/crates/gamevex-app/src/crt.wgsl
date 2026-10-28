#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var source_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var source_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> params: vec4<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var text_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var text_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<uniform> monitor: vec4<f32>;

fn sample_source(uv: vec2<f32>) -> vec3<f32> {
    return textureSample(source_texture, source_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0))).rgb;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    // Warp both layers together, preserving high-resolution text in Pixel CRT.
    // Input and browser secondary-click suppression use this identical transform.
    let p = mesh.uv * 2.0 - 1.0;
    let q = p * (1.0 + monitor.x * dot(p, p));
    let uv = q * 0.5 + 0.5;
    let edge = min(min(uv.x, 1.0-uv.x), min(uv.y, 1.0-uv.y));
    let coverage = smoothstep(0.0, 0.002, edge);
    let texel = 1.0 / vec2<f32>(textureDimensions(source_texture));
    var color = sample_source(uv);
    if params.z > 0.5 {
        if params.x > 0.0 {
            var neighborhood = vec3<f32>(0.0);
            neighborhood += sample_source(uv + texel * vec2<f32>(-1.0,-1.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(0.0,-1.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(1.0,-1.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(-1.0,0.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(1.0,0.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(-1.0,1.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(0.0,1.0));
            neighborhood += sample_source(uv + texel * vec2<f32>(1.0,1.0));
            color += max(neighborhood * 0.125 - vec3<f32>(0.52), vec3<f32>(0.0)) * params.x;
        }
        let row = floor(mesh.position.y);
        color *= 1.0 - clamp(params.y, 0.0, 0.8) * (0.5 + 0.5 * cos(row * 3.14159265));
    }
    // The transparent text target contains premultiplied RGB from alpha blending.
    let text = textureSample(text_texture, text_sampler, clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0)));
    color = text.rgb + color * (1.0 - text.a);
    let radial = clamp(dot(q,q)*0.5, 0.0, 1.0);
    color *= 1.0 - monitor.y * 0.7 * pow(radial,1.2);
    return vec4<f32>(color * coverage, 1.0);
}
