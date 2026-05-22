struct TileData {
    count: atomic<u32>,
    indices: array<u32, 256>,
}

@group(0) @binding(0) var<storage, read> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read> sizes: array<vec2<f32>>;
@group(1) @binding(0) var<storage, read_write> tiles: array<TileData>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let prim_index = global_id.x;
    let num_primitives = arrayLength(&positions);
    
    if (prim_index >= num_primitives) {
        return;
    }
    
    let pos = positions[prim_index];
    let size = sizes[prim_index];
    
    let min_pos = pos;
    let max_pos = pos + size;
    
    let tile_size = 16.0;
    
    // Compute tile range
    let min_tx = u32(max(0.0, floor(min_pos.x / tile_size)));
    let min_ty = u32(max(0.0, floor(min_pos.y / tile_size)));
    
    // Assuming max resolution 1280x720 -> 80x45 tiles
    let grid_width = 80u;
    let grid_height = 45u;
    
    let max_tx = min(grid_width - 1u, u32(max(0.0, floor(max_pos.x / tile_size))));
    let max_ty = min(grid_height - 1u, u32(max(0.0, floor(max_pos.y / tile_size))));
    
    for (var ty = min_ty; ty <= max_ty; ty++) {
        for (var tx = min_tx; tx <= max_tx; tx++) {
            let tile_idx = ty * grid_width + tx;
            
            // Atomic append
            let write_idx = atomicAdd(&tiles[tile_idx].count, 1u);
            if (write_idx < 256u) {
                tiles[tile_idx].indices[write_idx] = prim_index;
            }
        }
    }
}
