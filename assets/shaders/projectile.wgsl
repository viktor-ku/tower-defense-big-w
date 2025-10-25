#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_functions
#import bevy_pbr::mesh_view_bindings

struct ProjectileMaterialUniform {
    color: vec4<f32>,
    glow: f32,
    _pad: vec3<f32>,
};

@group(1) @binding(0)
var<uniform> projectile: ProjectileMaterialUniform;

@fragment
fn fragment(input: bevy_pbr::MeshFragmentInput) -> bevy_pbr::MeshFragmentOutput {
    var pbr = bevy_pbr::pbr_input(
        input,
        projectile.color.rgb,
        0.0,
        0.1,
        0.5,
        projectile.color.a,
        vec3<f32>(0.0),
    );

    let facing = max(dot(normalize(input.world_normal), vec3<f32>(0.0, 1.0, 0.0)), 0.0);
    let energy = (0.6 + pow(facing, 1.5) * 0.8) * projectile.glow;
    pbr.emissive = projectile.color.rgb * energy;
    pbr.alpha = projectile.color.a * (0.3 + pow(facing, 1.5) * 0.7);

    return bevy_pbr::pbr_fragment(pbr);
}

