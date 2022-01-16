#![allow(dead_code)]

use std::mem;
use bytemuck::{Pod, Zeroable};
use odc::Transform;

#[derive(Copy, Clone)]
pub struct InstanceInfo {
    pub transform: Transform,
}

impl InstanceInfo {
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }
}

unsafe impl Zeroable for InstanceInfo {}
unsafe impl Pod for InstanceInfo {}


#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }

    pub const fn position_offset() -> usize {
        0
    }

    pub const fn color_offset() -> usize {
        mem::size_of::<[f32; 4]>()
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}


pub fn triangle_mesh() -> (&'static [u8], &'static [u8]) {
    (
        bytemuck::cast_slice(&TRIANGLE_VERTICES),
        bytemuck::cast_slice(&TRIANGLE_INDICES),
    )
}

pub const TRIANGLE_VERTICES: [Vertex; 3] = [
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
];

pub const TRIANGLE_INDICES: [u32; 3] = [0, 1, 2];

pub fn rectangle_mesh() -> (&'static [u8], &'static [u8]) {
    (
        bytemuck::cast_slice(&RECTANGLE_VERTICES),
        bytemuck::cast_slice(&RECTANGLE_INDICES),
    )
}

pub const RECTANGLE_VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0, 1.0],
        color: [0.0, 0.0, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0, 1.0],
        color: [0.0, 1.0, 1.0, 1.0],
    },
];

pub const RECTANGLE_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

fn main() {}