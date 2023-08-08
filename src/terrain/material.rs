use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::*;

#[derive(AsBindGroup, Debug, Clone, TypePath, TypeUuid)]
#[uuid = "0e1d21c6-dd0b-4780-a774-9d6153786f61"]
pub struct TerrainMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub heightmap: Handle<Image>,
}

impl From<Handle<Image>> for TerrainMaterial {
    fn from(heightmap: Handle<Image>) -> Self {
        TerrainMaterial { heightmap }
    }
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}
