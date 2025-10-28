# Blender → Bevy Modeling and Export Guide (GLB / glTF 2.0)

This guide shows how to prepare and export assets from Blender for direct import into Bevy using GLB (binary glTF 2.0). It emphasizes predictable scale/orientation, PBR materials, and embedded textures for hassle-free loading.

## TL;DR
- Use GLB (binary glTF 2.0). Avoid Draco compression.
- 1 unit = 1 meter. Apply transforms. Y-up, facing -Z forward (handled by exporter).
- Principled BSDF with Metallic-Roughness workflow. Correct color spaces.
- Export with Tangents, Normals, UVs, and embedded images.
- Place files under `assets/models/<category>/<name>.glb`.
- Load with `asset_server.load("models/<category>/<name>.glb#Scene0")` into a `SceneBundle`.

## Blender project setup
- Units: Metric, Unit Scale 1.0 (1 m)
- Consistent real-world scale (e.g., rocks ~0.5–2 m)
- Smooth shading; mark sharp edges where needed
- Set origin to base-center for easy placement on terrain

## Mesh preparation
- Apply transforms: Ctrl+A → Location, Rotation, Scale (target scale = 1.0; rotation = 0°)
- Triangulate (via modifier, or let the exporter triangulate)
- UV unwrap and pack islands (avoid overlaps for PBR)
- Ensure normals are correct; enable Auto Smooth if using hard edges
- Generate tangents (needed for proper normal map shading)

## Materials and textures (PBR)
- Shader: Principled BSDF
- Base Color texture: sRGB
- Normal Map texture: Non-Color; use OpenGL normal maps (green channel positive)
- Metallic and Roughness textures: Non-Color (glTF uses Metallic-Roughness workflow)
- Ambient Occlusion (optional): Non-Color (usually separate map)
- Keep textures power-of-two in size; prefer embedded images in GLB

## Orientation and scale
- Model facing forward in Blender’s front view; exporter maps to glTF -Z forward, +Y up (matches Bevy)
- Keep assets true to real-world scale; verify against a 1 m reference cube
- Place origin at the asset’s base to simplify ground placement

## Export: glTF 2.0 (GLB) settings in Blender
- Format: GLB Binary
- Include: Selected Objects (for single-asset export)
- Transform: Apply Transform ON
- Geometry: Apply Modifiers ON; UVs ON; Normals ON; Tangents ON; Vertex Colors as needed
- Materials: Export as glTF materials (Principled BSDF)
- Animation: OFF (unless required)
- Images: Embed; Filter: Auto; Do NOT use Draco compression

## File organization
Place exported assets here:
- `assets/models/<category>/<name>.glb`

Examples:
- `assets/models/rocks/rock01.glb`
- `assets/models/rocks/rock02.glb`

## Importing in Bevy
```rust
// Load and spawn the default scene from a GLB file
commands.spawn(SceneBundle {
    scene: asset_server.load("models/rocks/rock01.glb#Scene0"),
    transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(1.0)),
    ..default()
});

// Optionally, load specific nodes/meshes if you've named them in Blender:
// asset_server.load("models/rocks/rock01.glb#<NodeName>")
// asset_server.load("models/rocks/rock01.glb#Mesh0/Primitive0")
```

Notes:
- Bevy supports glTF 2.0 out-of-the-box. No extra plugin required.
- Use named collections/nodes in Blender if you plan to address sub-scenes or nodes.

## Pre-export checklist
- Transforms applied; origin at base
- Scale correct (1 unit = 1 m)
- Normals/tangents correct; shading looks right
- PBR textures connected to Principled BSDF with correct color spaces
- No hidden/disabled modifiers that should be applied
- Textures embedded (GLB) or correctly referenced (if using JSON .gltf)

## Troubleshooting
- Too shiny/too flat: verify Metallic/Roughness textures and color spaces
- Normal map seams/inversion: ensure OpenGL normals; include Tangents; flip green channel only if the source uses DirectX
- Wrong scale/orientation: re-apply transforms; export with Apply Transform ON
- Missing textures: ensure images are embedded in GLB or paths are relative and present

## Version notes
- Tested with Blender 3.x/4.x glTF 2.0 exporter and Bevy 0.14+

## FAQ
- Why GLB over GLTF? GLB bundles everything in one file (robust, portable). JSON .gltf references external files which are easier to misplace.
- Why not OBJ? OBJ lacks PBR/animation metadata and is less reliable for modern material workflows.
- Is USDZ supported? Not by Bevy’s loader.
