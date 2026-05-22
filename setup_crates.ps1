mv src\shader.wgsl assets\shaders\
mv src\binning.wgsl assets\shaders\

Set-Content -Path crates\hades_math\Cargo.toml -Value @"
[package]
name = "hades_math"
version = "0.1.0"
edition = "2024"

[dependencies]
glam.workspace = true
"@

Set-Content -Path crates\hades_scene\Cargo.toml -Value @"
[package]
name = "hades_scene"
version = "0.1.0"
edition = "2024"

[dependencies]
hades_math = { path = "../hades_math" }
bytemuck.workspace = true
glam.workspace = true
"@

Set-Content -Path crates\hades_gpu\Cargo.toml -Value @"
[package]
name = "hades_gpu"
version = "0.1.0"
edition = "2024"

[dependencies]
hades_math = { path = "../hades_math" }
wgpu.workspace = true
bytemuck.workspace = true
"@

Set-Content -Path crates\hades_text\Cargo.toml -Value @"
[package]
name = "hades_text"
version = "0.1.0"
edition = "2024"

[dependencies]
hades_math = { path = "../hades_math" }
rustybuzz.workspace = true
fontdue.workspace = true
etagere.workspace = true
glam.workspace = true
"@

Set-Content -Path crates\hades_renderer\Cargo.toml -Value @"
[package]
name = "hades_renderer"
version = "0.1.0"
edition = "2024"

[dependencies]
hades_math = { path = "../hades_math" }
hades_scene = { path = "../hades_scene" }
hades_gpu = { path = "../hades_gpu" }
hades_text = { path = "../hades_text" }
wgpu.workspace = true
bytemuck.workspace = true
winit.workspace = true
glam.workspace = true
"@

Set-Content -Path crates\hades_platform\Cargo.toml -Value @"
[package]
name = "hades_platform"
version = "0.1.0"
edition = "2024"

[dependencies]
winit.workspace = true
wgpu.workspace = true
log.workspace = true
"@

Set-Content -Path crates\hades_core\Cargo.toml -Value @"
[package]
name = "hades_core"
version = "0.1.0"
edition = "2024"

[dependencies]
hades_math = { path = "../hades_math" }
hades_scene = { path = "../hades_scene" }
hades_renderer = { path = "../hades_renderer" }
hades_gpu = { path = "../hades_gpu" }
hades_platform = { path = "../hades_platform" }
hades_text = { path = "../hades_text" }
winit.workspace = true
wgpu.workspace = true
log.workspace = true
pollster.workspace = true
glam.workspace = true
"@

Set-Content -Path apps\viewer\Cargo.toml -Value @"
[package]
name = "viewer"
version = "0.1.0"
edition = "2024"

[dependencies]
hades_core = { path = "../../crates/hades_core" }
hades_math = { path = "../../crates/hades_math" }
hades_scene = { path = "../../crates/hades_scene" }
hades_platform = { path = "../../crates/hades_platform" }
log.workspace = true
env_logger.workspace = true
winit.workspace = true
glam.workspace = true
"@
