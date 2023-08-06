use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::{Fbm, Perlin};

#[derive(Clone, Debug)]
pub struct TerrainConfig {
    pub size: u32,
    pub scale: f64,
    pub noise: Fbm<Perlin>,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            size: 256,
            scale: 10.0,
            noise: Fbm::default(),
        }
    }
}

impl TerrainConfig {
    pub fn generate_mesh(&self) -> Mesh {
        shape::Plane {
            size: self.scale as f32,
            subdivisions: self.size,
        }
        .into()
    }

    pub fn generate_heightmap(&self) -> Image {
        let usize_size = self.size as usize;

        let data = PlaneMapBuilder::<Fbm<Perlin>, 2>::new(self.noise.clone())
            .set_size(usize_size, usize_size)
            .set_x_bounds(0.0, self.scale)
            .set_y_bounds(0.0, self.scale)
            .build()
            .iter()
            .map(|v| (v * 255.0).floor() as u8)
            .collect();

        Image::new(
            Extent3d {
                width: self.size,
                height: self.size,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::R8Unorm,
        )
    }
}
