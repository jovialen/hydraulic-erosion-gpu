mod material;
pub use material::*;

use bevy::prelude::*;

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

