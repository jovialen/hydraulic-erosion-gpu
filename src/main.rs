mod terrain;

use crate::terrain::{Terrain, TerrainMaterial};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::DirectionalLightBundle;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(WireframePlugin)
        .add_plugins(MaterialPlugin::<TerrainMaterial>::default())
        .insert_resource(WireframeConfig { global: false })
        .add_systems(Startup, (setup_camera, setup_terrain, setup_lights))
        .add_systems(Update, edit_settings)
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
            illuminance: 25_000.0,
            shadows_enabled: false,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 1000.0, 0.01).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    let terrain = Terrain {
        size: 256,
        scale: 10.0,
    };

    let heightmap = images.add(terrain.into());
    let mut material = TerrainMaterial::from(heightmap.clone());
    material.base_color_texture = Some(heightmap.clone());

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(terrain.into()),
        material: materials.add(material),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}

fn edit_settings(mut wf_config: ResMut<WireframeConfig>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::C) {
        wf_config.global = !wf_config.global;
    }
}
