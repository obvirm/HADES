use crate::renderer::Renderer;
use hades_scene::SceneSoA;

pub trait RenderPass {
    fn name(&self) -> &str;
    fn execute(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        renderer: &mut Renderer,
        scene: &SceneSoA,
    );
}

pub struct RenderGraph {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }
    
    pub fn add_pass(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }
    
    pub fn execute(
        &mut self,
        renderer: &mut Renderer,
        scene: &SceneSoA,
    ) -> Result<(), ()> {
        let surface_texture = match renderer.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => return Err(()),
            _ => return Ok(()),
        };
        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Graph Encoder"),
        });
        
        for pass in &mut self.passes {
            pass.execute(&mut encoder, &view, renderer, scene);
        }
        
        renderer.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
        
        renderer.advance_frames();
        
        Ok(())
    }
}
