use std::mem;

const VEC4_SIZE: u64 = mem::size_of::<[f32; 4]>() as _;
const MAT4_SIZE: u64 = VEC4_SIZE * 4;

pub mod color_mesh;
pub mod deferred;