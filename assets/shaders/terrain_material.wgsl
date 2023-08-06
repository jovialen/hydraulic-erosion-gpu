#import bevy_pbr::mesh_bindings      	mesh
#import bevy_pbr::mesh_vertex_output 	MeshVertexOutput
#import bevy_pbr::mesh_functions		as mesh_functions

@group(1) @binding(0)
var terrain_texture: texture_2d<f32>;
@group(1) @binding(1)
var terrain_sampler: sampler;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(3) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(4) color: vec4<f32>,
#endif
};

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    var out: MeshVertexOutput;

    var model = mesh.model;

#ifdef VERTEX_NORMALS
    out.world_normal = mesh_functions::mesh_normal_local_to_world(vertex.normal);
#endif

#ifdef VERTEX_POSITIONS
	var size = textureDimensions(terrain_texture, 0);
	var coords = vec2<u32>(vec2<f32>(size) * vertex.uv);
	var height = textureLoad(terrain_texture, coords, 0).r;
	
    out.world_position = mesh_functions::mesh_position_local_to_world(
		model,
		vec4<f32>(vertex.position.x, vertex.position.y + height, vertex.position.z, 1.0),
	);
    out.position = mesh_functions::mesh_position_world_to_clip(out.world_position);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    out.instance_index = vertex.instance_index;
#endif

    return out;
}
