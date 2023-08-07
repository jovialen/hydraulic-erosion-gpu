@group(0) @binding(0)
var texture: texture_storage_2d<r32float, read_write>;

@compute @workgroup_size(8, 8, 1)
fn erode(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	var location = invocation_id.xy;
	var height = textureLoad(texture, location);
	textureStore(texture, location, height);
}
