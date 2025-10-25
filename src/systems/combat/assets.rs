use bevy::math::primitives::{Circle, Rectangle, Sphere};
use bevy::prelude::*;

/// Cached meshes/materials for enemy health bars to avoid reallocations.
#[derive(Resource, Default)]
pub struct EnemyHealthBarAssets {
    quad_mesh: Option<Handle<Mesh>>,
    background_material: Option<Handle<StandardMaterial>>,
    fill_material: Option<Handle<StandardMaterial>>,
}

impl EnemyHealthBarAssets {
    pub fn mesh(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.quad_mesh
            .get_or_insert_with(|| meshes.add(build_quad_mesh()))
            .clone()
    }

    pub fn background_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.background_material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgba(0.05, 0.05, 0.05, 0.7),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone()
    }

    pub fn fill_material(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial> {
        self.fill_material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgba(0.2, 0.85, 0.2, 0.9),
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone()
    }
}

/// Shared meshes used by projectile/impact/explosion effects.
#[derive(Resource, Default)]
pub struct CombatVfxAssets {
    projectile_mesh: Option<Handle<Mesh>>,
    impact_mesh: Option<Handle<Mesh>>,
    explosion_mesh: Option<Handle<Mesh>>,
}

impl CombatVfxAssets {
    pub fn projectile_mesh(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.projectile_mesh
            .get_or_insert_with(|| meshes.add(Mesh::from(Sphere::new(0.25))))
            .clone()
    }

    pub fn impact_mesh(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.impact_mesh
            .get_or_insert_with(|| meshes.add(Mesh::from(Circle::new(0.9))))
            .clone()
    }

    pub fn explosion_mesh(&mut self, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.explosion_mesh
            .get_or_insert_with(|| meshes.add(Mesh::from(Sphere::new(0.6))))
            .clone()
    }
}

fn build_quad_mesh() -> Mesh {
    Mesh::from(Rectangle::new(1.0, 1.0))
}
