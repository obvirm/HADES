use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec4};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Rect {
    pub position: Vec2,
    pub size: Vec2,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct RoundedRect {
    pub position: Vec2,
    pub size: Vec2,
    pub radius: f32,
    pub _padding: f32, // alignment padding
}

// Structure of Arrays (SoA) layout for primitive data
// During traversal, we will extract parameters and push them into contiguous arrays.
pub struct SceneSoA {
    pub positions: Vec<Vec2>,
    pub sizes: Vec<Vec2>,
    pub colors: Vec<Vec4>,
    pub radii: Vec<f32>,
}

impl SceneSoA {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            sizes: Vec::new(),
            colors: Vec::new(),
            radii: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.sizes.clear();
        self.colors.clear();
        self.radii.clear();
    }
    
    pub fn push_rect(&mut self, rect: &Rect, color: Vec4) {
        self.positions.push(rect.position);
        self.sizes.push(rect.size);
        self.colors.push(color);
        self.radii.push(0.0);
    }

    pub fn push_rounded_rect(&mut self, rounded: &RoundedRect, color: Vec4) {
        self.positions.push(rounded.position);
        self.sizes.push(rounded.size);
        self.colors.push(color);
        self.radii.push(rounded.radius);
    }
}
