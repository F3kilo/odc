mod bind;
mod buffers;
mod samplers;
mod textures;

pub use bind::{BindGroups, BindGroupFactory};
pub use buffers::{Buffer, BufferInfo};
pub use samplers::Sampler;
use std::borrow::Borrow;
pub use textures::{Texture, TextureInfo};

use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Index;

pub struct Resources<Id: Hash + Eq> {
    pub buffers: Storage<Id, Buffer>,
    pub textures: Storage<Id, Texture>,
    pub samplers: Storage<Id, Sampler>,
}

pub struct Storage<Id, Val>(HashMap<Id, Val>);

impl<Id, Val> Default for Storage<Id, Val> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<Id: Hash + Eq, Val> FromIterator<(Id, Val)> for Storage<Id, Val> {
    fn from_iter<T: IntoIterator<Item = (Id, Val)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
}

impl<Id: Hash + Eq, Val> Storage<Id, Val> {
    pub fn insert(&mut self, id: Id, val: Val) {
        self.0.insert(id, val);
    }

    pub fn get<Q>(&self, id: &Q) -> &Val
    where
        Id: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.0.get(id.borrow()).unwrap()
    }

    pub fn replace(&mut self, id: Id, val: Val) -> Val {
        self.0.insert(id, val).unwrap()
    }
}

impl<Q, Id, Val> Index<&Q> for Storage<Id, Val>
where
    Id: Hash + Eq + Borrow<Q>,
    Q: Hash + Eq,
{
    type Output = Val;

    fn index(&self, index: &Q) -> &Self::Output {
        self.get(index)
    }
}

pub struct ResourceFactory<'a> {
    device: &'a wgpu::Device,
}

impl<'a> ResourceFactory<'a> {
    pub fn new(device: &'a wgpu::Device) -> Self {
        Self { device }
    }

    pub fn create_buffer(&self, info: BufferInfo) -> Buffer {
        let handle = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&info.name),
            size: info.size,
            usage: info.usage,
            mapped_at_creation: false,
        });

        Buffer { handle, info }
    }
}
