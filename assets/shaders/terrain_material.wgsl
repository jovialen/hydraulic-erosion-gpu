#import bevy_pbr::mesh_bindings      	mesh
#import bevy_pbr::mesh_vertex_output 	MeshVertexOutput
#import bevy_pbr::mesh_functions		as mesh_functions

@group(1) @binding(13)
var terrain_texture: texture_2d<f32>;
@group(1) @binding(14)
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

fn uv_to_coords(t: texture_2d<f32>, uv: vec2<f32>) -> vec2<u32> {
	var size = textureDimensions(t, 0);
    size.x -= 1u;
    size.y -= 1u;
	return vec2<u32>(vec2<f32>(size) * uv);
}

fn height_at(t: texture_2d<f32>, coords: vec2<u32>) -> f32 {
    var size = textureDimensions(t, 0);
    var coords = coords;
    
    if coords.x > size.x {
        coords.x = size.x - 1u;
    }
    if coords.y > size.y {
        coords.y = size.y - 1u;
    }
    
    return textureLoad(t, coords, 0).r;
}

@vertex
fn vertex(vertex: Vertex) -> MeshVertexOutput {
    var out: MeshVertexOutput;

    var model = mesh.model;

#ifdef VERTEX_POSITIONS
    var height = height_at(terrain_texture, uv_to_coords(terrain_texture, vertex.uv));
	
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

#ifdef VERTEX_NORMALS
    var coords = uv_to_coords(terrain_texture, vertex.uv);

    var dx = vec2<u32>(1u, 0u);
    var dy = vec2<u32>(0u, 1u);
    
    var up = height_at(terrain_texture, coords + dy);
    var down = height_at(terrain_texture, coords - dy);
    var right = height_at(terrain_texture, coords + dx);
    var left = height_at(terrain_texture, coords - dx);

    var normal = normalize(vec3<f32>(
        left - right,
        0.2,
        down - up,
    ));
    
    out.world_normal = mesh_functions::mesh_normal_local_to_world(normal);
#endif

    return out;
}

@fragment
fn fragment(mesh: MeshVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(mesh.world_normal, 1.0);
}
