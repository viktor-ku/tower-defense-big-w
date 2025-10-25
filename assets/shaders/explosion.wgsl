#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings

struct ExplosionMaterialUniform {
    color: vec4<f32>,
    progress: f32,
    glow: f32,
    _pad: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> explosion: ExplosionMaterialUniform;

fn ring_mask(position: vec3<f32>, progress: f32) -> f32 {
    let radius = progress * 2.5 + 0.1;
    let dist = length(position.xz);
    let shell = 1.0 - smoothstep(radius - 0.3, radius + 0.15, dist);
    let core = smoothstep(radius - 0.6, radius - 0.15, dist);
    let radial = mix(shell, core, 0.35);
    return clamp(radial, 0.0, 1.0);
}

fn flicker(intensity: f32, progress: f32) -> f32 {
    let pulse = sin(progress * 18.0) * 0.5 + 0.5;
    let surge = smoothstep(0.0, 0.2, progress);
    let decay = 1.0 - smoothstep(0.55, 1.0, progress);
    return intensity * mix(pulse, 1.0, surge) * decay;
}

@fragment
fn fragment(input: bevy_pbr::MeshFragmentInput) -> bevy_pbr::MeshFragmentOutput {
    var pbr = bevy_pbr::pbr_input(
        input,
        vec3<f32>(0.0),
        0.0,
        0.0,
        0.2,
        0.0,
        vec3<f32>(0.0),
    );

    let progress = clamp(explosion.progress, 0.0, 1.0);
    let mask = ring_mask(input.model_position, progress);
    let flicker_energy = flicker(explosion.glow, progress);

    let fire_color = vec3<f32>(1.1, 0.65, 0.25);
    let smoke_color = vec3<f32>(0.25, 0.25, 0.3);
    let color_mix = mix(fire_color, smoke_color, progress * 0.8);

    pbr.emissive = (explosion.color.rgb * 0.7 + color_mix * 0.9) * mask * flicker_energy;
    pbr.alpha = explosion.color.a * mask * (1.0 - progress * 0.4);

    return bevy_pbr::pbr_fragment(pbr);
}


