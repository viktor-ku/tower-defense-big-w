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
        vec3<f32>(1.0),
        0.0,
        0.05,
        0.4,
        projectile.color.a,
        vec3<f32>(0.0),
    );

    pbr.emissive = vec3<f32>(4.0 * projectile.glow);
    pbr.alpha = projectile.color.a;

    return bevy_pbr::pbr_fragment(pbr);
}

