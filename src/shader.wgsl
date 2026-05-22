struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate a fullscreen triangle:
    // vertex 0: (0, 0) -> clip (-1, 1)
    // vertex 1: (2, 0) -> clip (3, 1)
    // vertex 2: (0, 2) -> clip (-1, -3)
    let x = f32((in_vertex_index << 1u) & 2u);
    let y = f32(in_vertex_index & 2u);
    
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, 1.0 - y * 2.0, 0.0, 1.0);
    return out;
}

struct TileDataReadOnly {
    count: u32,
    indices: array<u32, 256>,
}

@group(0) @binding(0) var<storage, read> positions: array<vec2<f32>>;
@group(0) @binding(1) var<storage, read> sizes: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> colors: array<vec4<f32>>;
@group(0) @binding(3) var<storage, read> radii: array<f32>;

@group(1) @binding(0) var<storage, read> tiles: array<TileDataReadOnly>;

fn sd_round_box(p: vec2<f32>, b: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - b + vec2<f32>(r, r);
    return length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let frag_coord = in.clip_position.xy;
    
    let tile_size = 16.0;
    let grid_width = 80u;
    
    let tx = u32(frag_coord.x / tile_size);
    let ty = u32(frag_coord.y / tile_size);
    
    let tile_idx = ty * grid_width + tx;
    
    let count = tiles[tile_idx].count;
    let num_prims = min(count, 256u);
    
    var final_color = vec4<f32>(0.1, 0.2, 0.3, 1.0); // background
    
    for (var i = 0u; i < num_prims; i++) {
        let prim_idx = tiles[tile_idx].indices[i];
        
        let pos = positions[prim_idx];
        let size = sizes[prim_idx];
        let color = colors[prim_idx];
        let radius = radii[prim_idx];
        
        let half_size = size * 0.5;
        let center = pos + half_size;
        
        let p = frag_coord - center;
        
        let d = sd_round_box(p, half_size, radius);
        
        // Anti-aliasing via screen-space derivatives
        let fw = fwidth(d);
        // Avoid division by zero
        let alpha = clamp(0.5 - d / max(fw, 0.001), 0.0, 1.0);
        
        // Standard alpha blending: src over dst
        let out_alpha = color.a * alpha;
        let inv_alpha = 1.0 - out_alpha;
        
        final_color = vec4<f32>(
            (color.rgb * out_alpha) + (final_color.rgb * inv_alpha),
            out_alpha + final_color.a * inv_alpha
        );
    }
    
    return final_color;
}
