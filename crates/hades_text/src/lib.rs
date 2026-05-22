use rustybuzz::{Face, UnicodeBuffer};
use fontdue::Font;
use etagere::{AtlasAllocator, size2};
use std::collections::HashMap;

pub struct TextSystem {
    rustybuzz_face: Face<'static>,
    fontdue_font: Font,
    atlas_allocator: AtlasAllocator,
    glyph_cache: HashMap<u16, GlyphData>,
    pub atlas_texture_data: Vec<u8>,
    pub atlas_size: u32,
    pub atlas_dirty: bool,
}

#[derive(Clone, Copy)]
pub struct GlyphData {
    pub tex_x: f32,
    pub tex_y: f32,
    pub tex_w: f32,
    pub tex_h: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl TextSystem {
    pub fn new(font_data: &'static [u8]) -> Self {
        let rustybuzz_face = rustybuzz::Face::from_slice(font_data, 0).unwrap();
        let fontdue_font = Font::from_bytes(font_data, fontdue::FontSettings::default()).unwrap();
        let atlas_size = 1024;

        Self {
            rustybuzz_face,
            fontdue_font,
            atlas_allocator: AtlasAllocator::new(size2(atlas_size as i32, atlas_size as i32)),
            glyph_cache: HashMap::new(),
            atlas_texture_data: vec![0; (atlas_size * atlas_size) as usize],
            atlas_size,
            atlas_dirty: true,
        }
    }

    pub fn shape_text(&mut self, text: &str, size: f32) -> Vec<(GlyphData, glam::Vec2)> {
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(text);
        
        let glyph_buffer = rustybuzz::shape(&self.rustybuzz_face, &[], buffer);
        let mut result = Vec::new();
        
        let mut cursor_x = 0.0;
        
        for (info, pos) in glyph_buffer.glyph_infos().iter().zip(glyph_buffer.glyph_positions()) {
            let glyph_index = info.glyph_id as u16;
            
            if !self.glyph_cache.contains_key(&glyph_index) {
                let (metrics, bitmap) = self.fontdue_font.rasterize_indexed(glyph_index, size);
                
                if metrics.width > 0 && metrics.height > 0 {
                    let allocation = self.atlas_allocator.allocate(size2(metrics.width as i32, metrics.height as i32));
                    if let Some(alloc) = allocation {
                        let rect = alloc.rectangle;
                        let rx = rect.min.x as usize;
                        let ry = rect.min.y as usize;
                        
                        for y in 0..metrics.height {
                            for x in 0..metrics.width {
                                let atlas_idx = (ry + y) * self.atlas_size as usize + (rx + x);
                                self.atlas_texture_data[atlas_idx] = bitmap[y * metrics.width + x];
                            }
                        }
                        
                        self.atlas_dirty = true;
                        
                        self.glyph_cache.insert(glyph_index, GlyphData {
                            tex_x: rx as f32 / self.atlas_size as f32,
                            tex_y: ry as f32 / self.atlas_size as f32,
                            tex_w: metrics.width as f32 / self.atlas_size as f32,
                            tex_h: metrics.height as f32 / self.atlas_size as f32,
                            offset_x: metrics.xmin as f32,
                            offset_y: metrics.ymin as f32,
                        });
                    }
                }
            }
            
            if let Some(glyph_data) = self.glyph_cache.get(&glyph_index) {
                let current_pos = glam::Vec2::new(
                    cursor_x + pos.x_offset as f32 / 64.0,
                    pos.y_offset as f32 / 64.0
                );
                result.push((*glyph_data, current_pos));
            }
            
            cursor_x += pos.x_advance as f32 / 64.0;
        }
        
        result
    }
}
