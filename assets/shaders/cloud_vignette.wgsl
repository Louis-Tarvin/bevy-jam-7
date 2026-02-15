#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

struct DreamCloudSettings {
    // x = coverage, y = time, z = edge_softness, w = boundary_thickness
    boundary: vec4<f32>,
    // x = wobble_strength, y = wobble_frequency, z = wobble_speed
    wobble: vec4<f32>,
}

@group(0) @binding(2) var<uniform> settings: DreamCloudSettings;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(screen_texture, texture_sampler, in.uv);

    let coverage = clamp(settings.boundary.x, 0.0, 1.0);
    let time = settings.boundary.y;
    let edge_softness = max(settings.boundary.z, 0.01);
    let boundary_thickness = max(settings.boundary.w, 0.0);
    let wobble_strength = settings.wobble.x;
    let wobble_frequency = settings.wobble.y;
    let wobble_speed = settings.wobble.z;

    let centered_uv = in.uv * 2.0 - vec2<f32>(1.0, 1.0);
    let dist_to_center = length(centered_uv);
    let dir = centered_uv / max(dist_to_center, 0.0001);

    let w1 = sin(dot(dir, vec2<f32>(1.0, 0.0)) * wobble_frequency + time * wobble_speed);
    let w2 = sin(dot(dir, vec2<f32>(0.0, 1.0)) * wobble_frequency * 1.73 - time * wobble_speed * 1.29);
    let wobble = (w1 + w2 * 0.5) * wobble_strength;

    let open_radius = 1.45 + 1.5 * wobble_strength + boundary_thickness * 0.5 + edge_softness;
    let clear_radius = mix(open_radius, -0.25, coverage);
    let boundary_radius = clear_radius + wobble;
    let signed_distance = dist_to_center - boundary_radius;

    // pure white outside the boundary.
    let outside_mask = step(boundary_thickness * 0.5, signed_distance);
    let with_white_outside = mix(scene.rgb, vec3<f32>(1.0, 1.0, 1.0), outside_mask);

    // boundary ring.
    let line_distance = abs(signed_distance);
    let line_mask = 1.0 - smoothstep(
        boundary_thickness * 0.5,
        boundary_thickness * 0.5 + edge_softness,
        line_distance,
    );

    let color = mix(with_white_outside, vec3<f32>(1.0), line_mask);
    return vec4<f32>(color, scene.a);
}
