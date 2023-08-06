#import bevy_pbr::mesh_types          Mesh
#import bevy_pbr::mesh_bindings       mesh
#import bevy_pbr::mesh_vertex_output  MeshVertexOutput
#import bevy_pbr::mesh_functions as mesh_functions

@group(1) @binding(0)
var terrain_texture: texture_2d<f32>;
@group(1) @binding(1)
var terrain_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
	@location(0) position: vec3<f32>,
	@location(2) uv: vec2<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
	var out: MeshVertexOutput;

	var size = textureDimensions(terrain_texture, 0);
	var coords = vec2<u32>(vec2<f32>(size) * vertex.uv);
	var height = textureLoad(terrain_texture, coords, 0).r;
	
	out.position = mesh_functions::mesh_position_local_to_clip(
		mesh.model,
		vec4<f32>(vertex.position.x, vertex.position.y + height, vertex.position.z, 1.0),
	);
	
	return out;
}
