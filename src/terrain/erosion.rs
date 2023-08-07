use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_resource::*;
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::{Render, RenderApp, RenderSet};
use std::borrow::Cow;

const EROSION_NODE: &str = "erosion_node";
const EROSION_COMPUTE_SHADER: &str = "shaders/erosion.wgsl";
const WORKGROUP_SIZE: u32 = 1024;

pub struct ErosionPlugin;

impl Plugin for ErosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<ErosionImage>::default());
        app.add_systems(PostUpdate, update_erosion_image);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(EROSION_NODE, ErosionNode::default());
        render_graph.add_node_edge(EROSION_NODE, bevy::render::main_graph::node::CAMERA_DRIVER);
    }

    fn finish(&self, app: &mut App) {
        app.init_resource::<ErosionQueue>();
        app.init_resource::<ErosionImage>();

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<ErosionPipeline>();
    }
}

#[derive(Resource, Default)]
pub struct ErosionQueue(pub Vec<Handle<Image>>);

#[derive(Resource, Clone, Default, Deref, ExtractResource)]
struct ErosionImage(Option<Handle<Image>>);

#[derive(Resource)]
struct ErosionImageBindGroup(Option<BindGroup>);

fn update_erosion_image(mut queue: ResMut<ErosionQueue>, mut current: ResMut<ErosionImage>) {
    current.0 = queue.0.pop();
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ErosionPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    image: Res<ErosionImage>,
    render_device: Res<RenderDevice>,
) {
    commands.insert_resource(ErosionImageBindGroup(image.0.clone().map(|image| {
        let view = &gpu_images[&image];
        render_device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &pipeline.texture_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&view.texture_view),
            }],
        })
    })));
}

#[derive(Resource)]
struct ErosionPipeline {
    texture_bind_group_layout: BindGroupLayout,
    pipeline: CachedComputePipelineId,
}

impl FromWorld for ErosionPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Erosion Compute Pipeline Bind Group Layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });

        let shader = world.resource::<AssetServer>().load(EROSION_COMPUTE_SHADER);
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("Erosion Compute Pipeline")),
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: vec![],
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("erode"),
        });

        ErosionPipeline {
            texture_bind_group_layout,
            pipeline,
        }
    }
}

struct ErosionNode {
    ready: bool,
}

impl Default for ErosionNode {
    fn default() -> Self {
        Self { ready: false }
    }
}

impl bevy::render::render_graph::Node for ErosionNode {
    fn update(&mut self, world: &mut World) {
        if self.ready {
            return;
        }

        let pipeline = world.resource::<ErosionPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        self.ready = match pipeline_cache.get_compute_pipeline_state(pipeline.pipeline) {
            CachedPipelineState::Ok(_) => true,
            _ => false,
        };
    }

    fn run(
        &self,
        _graph: &mut bevy::render::render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), bevy::render::render_graph::NodeRunError> {
        if !self.ready {
            return Ok(());
        }

        let texture_bind_group = &world.resource::<ErosionImageBindGroup>().0;
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ErosionPipeline>();

        if let Some(bind_group) = texture_bind_group {
            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            pass.set_bind_group(0, bind_group, &[]);

            let erosion_pipeline = pipeline_cache
                .get_compute_pipeline(pipeline.pipeline)
                .expect("Compute pipeline should be ready");

            pass.set_pipeline(erosion_pipeline);

            info!("Dispatching compute erosion shader");
            pass.dispatch_workgroups(50000 / WORKGROUP_SIZE, 1, 1);
            // pass.dispatch_workgroups(10, 10, 1);
        }

        Ok(())
    }
}
