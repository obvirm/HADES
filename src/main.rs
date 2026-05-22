use log::info;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use std::sync::Arc;

mod memory;
mod scene;
mod text;
mod graph;
mod renderer;
use renderer::Renderer;

struct HadesApp<'a> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'a>>,
    graph: Option<graph::RenderGraph>,
    scene: scene::SceneSoA,
    text_system: Option<text::TextSystem>,
}

impl<'a> Default for HadesApp<'a> {
    fn default() -> Self {
        Self {
            window: None,
            renderer: None,
            graph: None,
            scene: scene::SceneSoA::new(),
            text_system: None,
        }
    }
}

impl<'a> ApplicationHandler for HadesApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("HADES Engine")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            let renderer = pollster::block_on(Renderer::new(window.clone()));
            self.renderer = Some(renderer);

            let mut graph = graph::RenderGraph::new();
            graph.add_pass(Box::new(renderer::ComputeBinningPass));
            graph.add_pass(Box::new(renderer::SdfEvaluationPass));
            self.graph = Some(graph);

            // Load a system font and leak it for 'static lifetime
            let font_bytes = std::fs::read("C:\\Windows\\Fonts\\arial.ttf").unwrap_or_else(|_| vec![]);
            if !font_bytes.is_empty() {
                let leaked_font: &'static [u8] = Box::leak(font_bytes.into_boxed_slice());
                self.text_system = Some(text::TextSystem::new(leaked_font));
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("Close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(physical_size);
                }
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(graph)) = (&mut self.renderer, &mut self.graph) {
                    self.scene.clear();
                    
                    self.scene.push_rect(
                        &scene::Rect {
                            position: glam::Vec2::new(50.0, 50.0),
                            size: glam::Vec2::new(100.0, 100.0),
                        },
                        glam::Vec4::new(1.0, 0.0, 0.0, 1.0),
                    );

                    self.scene.push_rounded_rect(
                        &scene::RoundedRect {
                            position: glam::Vec2::new(100.0, 100.0),
                            size: glam::Vec2::new(150.0, 80.0),
                            radius: 20.0,
                            _padding: 0.0,
                        },
                        glam::Vec4::new(0.0, 1.0, 0.0, 0.8),
                    );

                    if let Some(text_sys) = &mut self.text_system {
                        let glyphs = text_sys.shape_text("HADES Engine - Procedural SDF", 32.0);
                        // For now we just prove text_sys is not dead code and shaping works.
                        // We will render it properly later.
                    }

                    renderer.upload_scene(&self.scene);
                    let _ = graph.execute(renderer, &self.scene);
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = HadesApp::default();
    event_loop.run_app(&mut app).unwrap();
}
