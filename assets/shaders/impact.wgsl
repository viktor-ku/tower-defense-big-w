#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings

struct ImpactMaterialUniform {
    color: vec4<f32>,
    progress: f32,
    _pad: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> impact: ImpactMaterialUniform;

fn radial_mask(model_pos: vec3<f32>, progress: f32) -> f32 {
    let radius = max(progress, 0.001);
    let dist = length(model_pos.xz);
    let leading = smoothstep(radius - 0.25, radius, dist);
    let fade = clamp(1.0 - progress, 0.0, 1.0);
    let rim = 1.0 - smoothstep(radius, radius + 0.2, dist);
    return clamp((rim * fade) + (1.0 - leading) * 0.15, 0.0, 1.0);
}

@fragment
fn fragment(input: bevy_pbr::MeshFragmentInput) -> bevy_pbr::MeshFragmentOutput {
    var pbr = bevy_pbr::pbr_input(
        input,
        vec3<f32>(0.0),
        0.0,
        0.0,
        0.5,
        0.0,
        vec3<f32>(0.0),
    );

    let mask = radial_mask(input.model_position, impact.progress);
    pbr.emissive = impact.color.rgb * mask * 1.2;
    pbr.alpha = impact.color.a * mask;

    return bevy_pbr::pbr_fragment(pbr);
}

