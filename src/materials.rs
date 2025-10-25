use bevy::math::{Vec2, Vec3, Vec4};
use bevy::pbr::Material;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy_shader::ShaderRef;

use bevy::prelude::AlphaMode;

/// Uniform data for projectile materials controlling color and glow intensity.
#[derive(Clone, Copy, ShaderType, Default, Debug)]
pub struct ProjectileMaterialUniform {
    pub color: Vec4,
    pub glow: f32,
    pub _pad: Vec3,
}

/// Simple glowing material used for tower projectiles.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ProjectileMaterial {
    #[uniform(0)]
    pub data: ProjectileMaterialUniform,
}

// No constructor is provided; this material is not currently used directly.

impl Material for ProjectileMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/projectile.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        if self.data.color.w >= 0.999 {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }
    }
}

/// Uniform data for radial impact flashes.
#[derive(Clone, Copy, ShaderType, Default, Debug)]
pub struct ImpactMaterialUniform {
    pub color: Vec4,
    pub progress: f32,
    pub _pad: Vec3,
}

/// Expanding radial flash material for tower impact effects.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ImpactMaterial {
    #[uniform(0)]
    pub data: ImpactMaterialUniform,
}

impl ImpactMaterial {
    pub fn new(color: Color) -> Self {
        let linear = color.to_linear();
        let rgba = linear.to_f32_array();
        ImpactMaterial {
            data: ImpactMaterialUniform {
                color: Vec4::from_array(rgba),
                progress: 0.0,
                _pad: Vec3::ZERO,
            },
        }
    }
}

impl Material for ImpactMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/impact.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

/// Uniform data for volumetric explosion effects.
#[derive(Clone, Copy, ShaderType, Default, Debug)]
pub struct ExplosionMaterialUniform {
    pub color: Vec4,
    pub progress: f32,
    pub glow: f32,
    pub _pad: Vec2,
}

/// Expanding explosion material used for lethal blasts.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ExplosionMaterial {
    #[uniform(0)]
    pub data: ExplosionMaterialUniform,
}

impl ExplosionMaterial {
    pub fn new(color: Color) -> Self {
        let linear = color.to_linear();
        let rgba = linear.to_f32_array();
        ExplosionMaterial {
            data: ExplosionMaterialUniform {
                color: Vec4::from_array(rgba),
                progress: 0.0,
                glow: 1.0,
                _pad: Vec2::ZERO,
            },
        }
    }
}

impl Material for ExplosionMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/explosion.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Add
    }
}
