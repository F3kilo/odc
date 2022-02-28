use bytemuck::{Pod, Zeroable};

#[derive(Copy, Clone)]
pub struct ColorVertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

unsafe impl Zeroable for ColorVertex {}
unsafe impl Pod for ColorVertex {}

#[derive(Copy, Clone)]
pub struct SpriteVertex {
    pub position: [f32; 4],
    pub uvs: [f32; 4],
}

unsafe impl Zeroable for SpriteVertex {}
unsafe impl Pod for SpriteVertex {}

pub fn triangle_mesh() -> (&'static [ColorVertex], &'static [u32]) {
    (&TRIANGLE_VERTICES, &TRIANGLE_INDICES)
}

pub const TRIANGLE_VERTICES: [ColorVertex; 3] = [
    ColorVertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [0.0, 1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

pub const TRIANGLE_INDICES: [u32; 3] = [0, 1, 2];

pub fn rectangle_mesh() -> (&'static [ColorVertex], &'static [u32]) {
    (&RECTANGLE_VERTICES, &RECTANGLE_INDICES)
}

pub const RECTANGLE_VERTICES: [ColorVertex; 4] = [
    ColorVertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [-1.0, 1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    ColorVertex {
        position: [1.0, 1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
    ColorVertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 1.0, 1.0],
    },
];

pub const RECTANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub fn sprite_mesh() -> (&'static [SpriteVertex], &'static [u32]) {
    (&SPRITE_VERTICES, &SPRITE_INDICES)
}

pub const SPRITE_VERTICES: [SpriteVertex; 4] = [
    SpriteVertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        uvs: [0.0, 1.0, 0.0, 0.0],
    },
    SpriteVertex {
        position: [-1.0, 1.0, 0.0, 1.0],
        uvs: [0.0, 0.0, 0.0, 0.0],
    },
    SpriteVertex {
        position: [1.0, 1.0, 0.0, 1.0],
        uvs: [1.0, 0.0, 1.0, 0.0],
    },
    SpriteVertex {
        position: [1.0, -1.0, 0.0, 1.0],
        uvs: [1.0, 1.0, 1.0, 0.0],
    },
];

pub const SPRITE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub fn skybox_mesh() -> (&'static [VertexPosition], &'static [u32]) {
    (&SKYBOX_VERTICES, &SKYBOX_INDICES)
}

pub type VertexPosition = [f32; 4];

const SKYBOX_VERTICES: [VertexPosition; 8] = [
    [-1.0, 1.0, 1.0, 0.0],   // Front top left
    [1.0, 1.0, 1.0, 0.0],    // Front top right
    [-1.0, -1.0, 1.0, 0.0],  // Front bottom left
    [1.0, -1.0, 1.0, 0.0],   // Front bottom right
    [-1.0, 1.0, -1.0, 0.0],  // Back top left
    [1.0, 1.0, -1.0, 0.0],   // Back top right
    [-1.0, -1.0, -1.0, 0.0], // Back bottom left
    [1.0, -1.0, -1.0, 0.0],  // Back bottom right
];

#[rustfmt::skip]
const SKYBOX_INDICES: [u32; 36] = [
    0, 1, 2,  // front
    1, 3, 2, 
      
    4, 6, 5,  // back
    5, 6, 7, 
      
    0, 5, 1,  // top
    0, 4, 5, 
          
    2, 7, 6,  // bottom
    2, 3, 7, 
          
    0, 2, 4,  // left
    4, 2, 6, 
          
    1, 5, 3,  // right
    5, 7, 3,     
];
