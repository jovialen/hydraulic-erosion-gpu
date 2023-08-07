mod terrain;

use crate::terrain::{ErosionPlugin, ErosionQueue, TerrainConfig, TerrainMaterial};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<TerrainMaterial>::default())
        .add_plugins((LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin))
        .add_plugins(ErosionPlugin)
        .add_systems(Startup, (setup_camera, setup_terrain, setup_lights))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

fn setup_lights(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10_000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(500.0, 1000.0, 500.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut erosion_queue: ResMut<ErosionQueue>,
) {
    let mut config = TerrainConfig {
        size: 512,
        scale: 10.0,
        ..default()
    };

    config.noise.frequency = 0.3;

    let heightmap = images.add(config.generate_heightmap());
    let mut material = TerrainMaterial::from(heightmap.clone());
    material.base_color_texture = Some(heightmap.clone());
    material.reflectance = 0.1;
    material.perceptual_roughness = 0.9;

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(config.generate_mesh()),
        material: materials.add(material),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    erosion_queue.0.push(heightmap);
}
