mod terrain;

use crate::terrain::{ErosionPlugin, ErosionQueue, TerrainConfig, TerrainMaterial};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

#[derive(Resource, Default)]
struct TerrainHeightmapImage(Handle<Image>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hydraulic Erosion".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(MaterialPlugin::<TerrainMaterial>::default())
        .add_plugins((LogDiagnosticsPlugin::default(), FrameTimeDiagnosticsPlugin))
        .add_plugins(EguiPlugin)
        .add_plugins(ErosionPlugin)
        .add_systems(Startup, (setup_camera, setup_terrain, setup_lights))
        .add_systems(Update, ui_controls)
        .init_resource::<TerrainHeightmapImage>()
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
    mut heightmap: ResMut<TerrainHeightmapImage>,
) {
    let mut config = TerrainConfig {
        size: 512,
        scale: 10.0,
        ..default()
    };

    config.noise.frequency = 0.2;

    heightmap.0 = images.add(config.generate_heightmap());
    let mut material = TerrainMaterial::from(heightmap.0.clone());
    material.base_color_texture = Some(heightmap.0.clone());
    material.reflectance = 0.1;
    material.perceptual_roughness = 0.9;

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(config.generate_mesh()),
        material: materials.add(material),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
}

fn ui_controls(
    mut contexts: EguiContexts,
    mut erosion_queue: ResMut<ErosionQueue>,
    heightmap: Res<TerrainHeightmapImage>,
) {
    egui::Window::new("Erosion Controls").show(contexts.ctx_mut(), |ui| {
        if ui.button("Simulate Erosion").clicked() {
            erosion_queue.0.push(heightmap.0.clone())
        }
    });
}
