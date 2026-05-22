# HADES Engine Master Plan

## Hybrid Analytic Deferred Evaluation System

> Procedural-field-based 2D rendering engine designed for GPU-first evaluation, tile-local execution, and scalable vector rendering.

---

# 1. Vision

HADES is not a traditional raster 2D renderer.

Traditional engines:

* tessellate paths
* generate triangles
* rasterize geometry

HADES:

* represents primitives as mathematical fields
* evaluates locally per tile
* minimizes geometry generation
* shifts complexity from CPU orchestration to GPU evaluation

Target domains:

* infinite canvas
* CAD/UI overlays
* node graph editors
* vector-heavy applications
* procedural interfaces
* zoom-intensive systems

Not intended as:

* universal browser engine
* DOM replacement
* generic desktop UI toolkit

---

# 2. Core Philosophy

Traditional Pipeline:

```text
Vector Paths
 -> Tessellation
 -> Triangle Buffers
 -> Rasterization
```

HADES Pipeline:

```text
Scene Graph
 -> Procedural Field Representation
 -> Tile Binning
 -> Local Evaluation
 -> Deferred Composition
```

---

# 3. Architectural Goals

## Primary Goals

* Minimize CPU rendering overhead
* Eliminate tessellation hot paths
* Reduce draw call pressure
* Scale efficiently with vector complexity
* Preserve infinite zoom sharpness
* Maximize GPU locality

## Secondary Goals

* Hybrid text rendering
* Render graph driven execution
* Multi-backend rendering
* Deterministic frame scheduling

---

# 4. Engine Architecture

# CPU Responsibilities

CPU acts as:

* scene compiler
* memory scheduler
* text shaper
* render graph builder

CPU does NOT:

* rasterize paths
* generate meshes dynamically
* rebuild geometry every frame

---

# GPU Responsibilities

GPU acts as:

* tile evaluator
* field processor
* compositor
* texture resolver

GPU executes:

* compute tile binning
* SDF evaluation
* compositing
* blending
* texture sampling

---

# 5. Mathematical Foundation

# 5.1 Signed Distance Functions (SDF)

All procedural primitives are represented analytically.

Examples:

* circles
* rounded rectangles
* lines
* shadows
* gradients
* Bézier curves

---

## Circle SDF

f(x,y)=\sqrt{(x-c_x)^2+(y-c_y)^2}-r

---

## Rounded Rectangle SDF

d=\operatorname{length}(\max(|p|-b+r,0))-r

---

# 5.2 Anti-Aliasing

Hardware derivative based anti-aliasing:

\alpha=\operatorname{clamp}(0.5-d/fwidth(d),0,1)

Advantages:

* no MSAA dependency
* bandwidth efficient
* resolution independent

---

# 5.3 Spatial Encoding

Use Morton Codes for:

* tile locality
* cache coherence
* compact traversal

Transformation:

```text
(x,y)
 ->
bit interleave
 ->
morton index
```

---

# 5.4 Transform System

CPU:

* affine 3×2 transforms

GPU:

* aligned 4×4 matrices

Reason:

* vec4 alignment
* coalesced GPU memory access
* SIMD efficiency

---

# 6. Memory Architecture

# 6.1 Frame Arena Allocator

No runtime heap allocations during rendering.

Use:

* linear bump allocator
* frame-local memory arenas

Per frame:

```text
reset pointer -> reuse memory
```

Complexity:

* O(1)

---

# 6.2 Triple Buffering

```text
Frame N     -> GPU execution
Frame N+1   -> CPU scene build
Frame N+2   -> upload staging
```

Goal:

* avoid CPU/GPU stalls

---

# 6.3 Structure of Arrays (SoA)

Avoid:

```rust
Vec<Shape>
```

Use:

```text
positions[]
sizes[]
colors[]
radii[]
transform_ids[]
```

Benefits:

* contiguous reads
* SIMD friendly
* GPU cache efficient

---

# 7. Render Graph System

# Directed Acyclic Graph (DAG)

Render graph manages:

* pass ordering
* resource lifetimes
* dependencies
* synchronization

Node Structure:

```text
RenderPassNode
    dependencies[]
    resources[]
    outputs[]
```

Pass Types:

* compute binning
* primitive evaluation
* blur
* composite
* atlas upload
* post processing

---

# 8. Rendering Pipeline

# Pass 1 — Scene Upload

CPU:

* flatten scene graph
* pack SSBO data
* upload parameters

---

# Pass 2 — Compute Tile Binning

Compute shader:

```text
for primitive:
    compute bounds
    determine overlapped tiles
    append primitive index
```

Output:

```text
Tile -> primitive lists
```

---

# Pass 3 — Tile Evaluation

Per tile:

```text
load primitive list
evaluate SDF
resolve coverage
blend output
```

No global primitive traversal.

---

# Pass 4 — Hybrid Text Rendering

Text pipeline:

* shaping on CPU
* rasterized atlas cache
* GPU texture fetch

No fully analytic font rendering.

Reason:

* performance stability
* cross-platform consistency
* lower shader complexity

---

# Pass 5 — Deferred Composition

Final compositing:

* blur
* opacity
* clipping
* post effects

---

# 9. Core Algorithms

# 9.1 Tile Binning

Goal:
avoid per-pixel global primitive evaluation.

Instead:

```text
pixel
 -> tile
 -> local primitives only
```

---

# 9.2 Lock-Free Append

GPU compute:

* atomic counters
* prefix sums
* compact writes

Avoid:

* mutexes
* synchronization contention

---

# 9.3 Local Field Evaluation

Each tile evaluates:

* only nearby primitives
* local compositing
* local blending

Improves:

* cache locality
* scalability

---

# 9.4 Hybrid Evaluation Path

```text
if geometry:
    evaluate SDF

if text/image:
    texture fetch
```

Reduces:

* ALU saturation
* shader divergence

---

# 10. Technology Stack

Language:

* Rust

GPU API:

* wgpu

Windowing:

* winit

Math:

* glam

Text shaping:

* rustybuzz

Font rasterization:

* fontdue

Zero-copy transfers:

* bytemuck

---

# 11. Multi-Backend Strategy

# Tier 1 — Full GPU Compute

Features:

* compute binning
* procedural evaluation
* async composition

Target:

* Vulkan
* Metal
* DX12
* WebGPU

---

# Tier 2 — Hybrid GPU

Reduced:

* tile complexity
* primitive limits
* procedural passes

Target:

* weaker mobile GPUs

---

# Tier 3 — CPU Fallback

Software raster backend.

Target:

* embedded systems
* VM rendering
* unsupported hardware

---

# 12. Optimization Strategy

# GPU Profiling Targets

Track:

* occupancy
* divergence
* bandwidth usage
* cache misses
* synchronization stalls

Tools:

* RenderDoc
* Tracy
* PIX

---

# 13. Development Roadmap

# Phase 1 — GPU Bootstrap

Goals:

* window creation
* swapchain
* fullscreen triangle
* render loop

---

# Phase 2 — Memory System

Implement:

* arena allocator
* upload manager
* triple buffering

---

# Phase 3 — Primitive Rendering

Implement:

* rects
* rounded rects
* gradients
* strokes

---

# Phase 4 — Compute Tile Binning

Implement:

* Morton tile indexing
* primitive binning
* compact tile lists

---

# Phase 5 — Hybrid Text System

Implement:

* shaping
* atlas cache
* glyph eviction
* subpixel rendering

---

# Phase 6 — Render Graph

Implement:

* DAG scheduling
* transient resources
* synchronization barriers

---

# Phase 7 — Optimization

Focus:

* cache locality
* occupancy
* ALU efficiency
* memory bandwidth

---

# 14. Risks

Major risks:

* GPU divergence
* bandwidth saturation
* mobile thermal throttling
* synchronization bugs
* atlas fragmentation
* shader debugging complexity

This architecture trades:

* CPU complexity
  for:
* GPU compute complexity

---

# 15. Strategic Advantage

Potential strengths over traditional renderers:

* infinite zoom sharpness
* reduced tessellation overhead
* scalable procedural rendering
* GPU-native evaluation
* efficient vector-heavy scenes

Potential weaknesses:

* heavy text workloads
* blur-heavy UIs
* low-end mobile GPUs
* debugging complexity

---

# 16. Final Direction

HADES should evolve as:

* a specialized procedural renderer
* not a universal UI replacement

Its strength is not:

> replacing every renderer

Its strength is:

> solving rendering workloads where traditional raster pipelines scale poorly.

Which is how genuinely interesting graphics systems usually survive. Not by becoming universal immediately, but by becoming absurdly good at one difficult thing first.
