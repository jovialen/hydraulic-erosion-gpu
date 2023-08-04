use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Terrain {
    pub size: u32,
    pub scale: f32,
}

impl From<Terrain> for Mesh {
    fn from(terrain: Terrain) -> Self {
        shape::Plane {
            size: terrain.scale,
            subdivisions: terrain.size,
        }
        .into()
    }
}

impl From<Terrain> for Image {
    fn from(terrain: Terrain) -> Self {
        let usize_size = terrain.size as usize;
        let f64_scale = terrain.scale as f64;

        let fbm: Fbm<Perlin> = Fbm::default();

        let data = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(fbm)
            .set_size(usize_size, usize_size)
            .set_x_bounds(0.0, f64_scale)
            .set_y_bounds(0.0, f64_scale)
            .build()
            .iter()
            .map(|v| (v * 255.0).floor() as u8)
            .collect();

        Image::new(
            Extent3d {
                width: terrain.size,
                height: terrain.size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::R8Unorm,
        )
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "0e1d21c6-dd0b-4780-a774-9d6153786f61"]
pub struct TerrainMaterial {
    #[texture(0)]
    #[sampler(1)]
    heightmap: Handle<Image>,
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}
