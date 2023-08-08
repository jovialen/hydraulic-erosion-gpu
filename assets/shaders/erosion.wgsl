@group(0) @binding(0)
var texture: texture_storage_2d<r32float, read_write>;

fn hash(seed: u32) -> u32 {
    var state = seed;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn random_float(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967295.0;
}

fn height_at(coords: vec2<u32>) -> f32 {
	return textureLoad(texture, coords).r;
}

fn set_height_at(coords: vec2<u32>, value: f32) {
	textureStore(texture, coords, vec4<f32>(value, 0.0, 0.0, 1.0));
}

struct Drop {
	position: vec2<f32>,
	direction: vec2<f32>,
	velocity: f32,
	water: f32,
	sediment: f32,
};

fn new_drop(pos: vec2<f32>) -> Drop {
	var drop: Drop;

	drop.position  = pos;
	drop.direction = vec2<f32>(0.0, 0.0);
	drop.velocity  = 1.0;
	drop.water     = 1.0;
	drop.sediment  = 0.0;

	return drop;
}

fn new_random_drop(seed: u32) -> Drop {
	var size = vec2<f32>(textureDimensions(texture));
	var x = max(random_float(seed * 62143262u) * size.x - 1.1, 0.1);
	var y = max(random_float(seed * 82734321u) * size.y - 1.1, 0.1);
	var pos = vec2<f32>(x, y);
	return new_drop(pos);
}

fn drop_coords(drop: Drop) -> vec2<u32> {
	return vec2<u32>(drop.position);
}

fn drop_uv(drop: Drop) -> vec2<f32> {
	var stripped = vec2<f32>(drop_coords(drop));
	return drop.position - stripped;
}

fn drop_out_of_bounds(drop: Drop) -> bool {
	var size = vec2<f32>(textureDimensions(texture));
	return 
		drop.position.x <  0.0            ||
		drop.position.x >= (size.x - 1.0) ||
		drop.position.y <  0.0            ||
		drop.position.y >= (size.y - 1.0) ;
}

fn drop_slope_gradient(drop: Drop) -> vec2<f32> {
	var coords = drop_coords(drop);
	var uv = drop_uv(drop);
	
	var tl = height_at(coords);
	var tr = height_at(coords + vec2<u32>(1u, 0u));
	var bl = height_at(coords + vec2<u32>(0u, 1u));
	var br = height_at(coords + vec2<u32>(1u, 1u));

	var gradient = vec2<f32>(
		((tr - tl) * (1.0 - uv.y)) + ((br - bl) * uv.y),
		((bl - tl) * (1.0 - uv.x)) + ((br - tr) * uv.x),
	);
	return gradient;
}

fn drop_height(drop: Drop) -> f32 {
	var coords = drop_coords(drop);
	var uv = drop_uv(drop);

	return
		height_at(coords)                     * (1.0 - uv.x) * (1.0 - uv.y) +
		height_at(coords + vec2<u32>(1u, 0u)) * uv.x         * (1.0 - uv.y) +
		height_at(coords + vec2<u32>(0u, 1u)) * (1.0 - uv.x) * uv.y         +
		height_at(coords + vec2<u32>(1u, 1u)) * uv.x         * uv.y         ;
}

fn deposit_terrain(drop: Drop, amount: f32) {
	var coords = drop_coords(drop);
	var uv = drop_uv(drop);

	var tl = height_at(coords);
	var tr = height_at(coords + vec2<u32>(1u, 0u));
	var bl = height_at(coords + vec2<u32>(0u, 1u));
	var br = height_at(coords + vec2<u32>(1u, 1u));

	set_height_at(coords                    , tl + (amount * (1.0 - uv.x) * (1.0 - uv.y)));
	set_height_at(coords + vec2<u32>(1u, 0u), tr + (amount * uv.x         * (1.0 - uv.y)));
	set_height_at(coords + vec2<u32>(0u, 1u), bl + (amount * (1.0 - uv.x) * uv.y        ));
	set_height_at(coords + vec2<u32>(1u, 1u), br + (amount * uv.x         * uv.y        ));
}

fn erode_terrain(drop: Drop, radius: i32, amount: f32) -> f32 {
	var size = vec2<i32>(textureDimensions(texture));
	var coords = vec2<i32>(drop_coords(drop));

	var weight_sum = 0.0;

	for (var x: i32 = -radius; x <= radius; x++) {
		var weight_coord: vec2<i32>;
		weight_coord.x = coords.x + x;
		
		if weight_coord.x < 0 || weight_coord.x >= size.x {
			continue;
		}
	
		for (var y: i32 = -radius; y <= radius; y++) {
			weight_coord.y = coords.y + y;
			
			if (weight_coord.y < 0 || weight_coord.y >= size.y) {
				continue;
			}

			var dist = distance(drop.position, vec2<f32>(weight_coord));
			var weight = max(f32(radius) - dist, 0.0);
			weight_sum += weight;
		}
	}

	var eroded = 0.0;
	
	for (var x: i32 = -radius; x <= radius; x++) {
		var weight_coord: vec2<i32>;
		weight_coord.x = coords.x + x;
		
		if weight_coord.x < 0 || weight_coord.x >= size.x {
			continue;
		}
	
		for (var y: i32 = -radius; y <= radius; y++) {
			weight_coord.y = coords.y + y;
			
			if (weight_coord.y < 0 || weight_coord.y >= size.y) {
				continue;
			}

			var height = height_at(vec2<u32>(weight_coord));
			var dist = distance(drop.position, vec2<f32>(weight_coord));
			var weight = max(f32(radius) - dist, 0.0);
			var weighted_amount = amount * (weight / weight_sum);
			var to_erode = min(height, weighted_amount);
			
			set_height_at(vec2<u32>(weight_coord), height - to_erode);
			eroded += to_erode;
		}
	}

	return eroded;
}

@compute @workgroup_size(1024, 1, 1)
fn erode(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
	var GRAVITY = 4.0;
	var EVAPORATION = 0.05;
	var EROSION = 0.3;
	var DEPOSITION = 0.3;
	var INERTIA = 0.05;
	var RADIUS = 3;
	var CAPACITY = 4.0;
	var MIN_CAPACITY = 0.01;
	var DROP_LIFETIME = 50u;

	var seed = invocation_id.y * num_workgroups.x + invocation_id.x;
	var drop = new_random_drop(seed);

	for (var i: u32 = 0u; i < DROP_LIFETIME; i++) {
		var gradient = drop_slope_gradient(drop);
		
		// Update the direction of the droplet
		drop.direction.x = (drop.direction.x * INERTIA) - (gradient.x * (1.0 - INERTIA));
		drop.direction.y = (drop.direction.y * INERTIA) - (gradient.y * (1.0 - INERTIA));
		drop.direction = normalize(drop.direction);

		// Stop simulating the droplet if it stops moving
		if drop.direction.x == 0.0 && drop.direction.y == 0.0 {
			break;
		}

		// Move the droplet
		var old_drop = new_drop(drop.position);
		drop.position += drop.direction;

		// Stop simulating the droplet if it leaves the map
		if drop_out_of_bounds(drop) {
			break;
		}

		// Calculate the capacity of the droplet
		var delta_height = drop_height(drop) - drop_height(old_drop);
		var raw_capacity = (-delta_height) * drop.velocity * drop.water * CAPACITY;
		var capacity = max(raw_capacity, MIN_CAPACITY);

		
		if delta_height > 0.0 {
			// Equalize height
			var deposit = min(drop.sediment, delta_height);
			drop.sediment -= deposit;
			deposit_terrain(old_drop, deposit);
		} else if drop.sediment > capacity {
			// Deposit sediment
			var deposit = (drop.sediment - capacity) * DEPOSITION;
			drop.sediment -= deposit;
			deposit_terrain(old_drop, deposit);
		} else {
			// Erode terrain
			var erode = min((capacity - drop.sediment) * EROSION, -delta_height);
			drop.sediment += erode_terrain(old_drop, RADIUS, erode);
		}

		// Update velocity and water content
		drop.velocity = sqrt(drop.velocity * drop.velocity + delta_height * GRAVITY);
		drop.water = drop.water * (1.0 - EVAPORATION);
	}
}
