#![allow(dead_code)]

use odc_core::model::*;
use std::collections::HashMap;
use std::mem;

const VEC4_SIZE: u64 = mem::size_of::<[f32; 4]>() as _;
const MAT4_SIZE: u64 = VEC4_SIZE * 4;

pub fn color_mesh_model() -> RenderModel {
    let vertex_buffer_name = "vertex";
    let vertex_buffer = Buffer { size: 2u64.pow(16) };
    let instance_buffer_name = "instance";
    let instance_buffer = Buffer { size: 2u64.pow(16) };
    let index_buffer_name = "index";
    let index_buffer = Buffer { size: 2u64.pow(16) };
    let uniform_buffer_name = "uniform";
    let uniform_size = MAT4_SIZE as u64 * 2;
    let uniform_buffer = Buffer { size: uniform_size };

    let buffers = HashMap::from_iter([
        (vertex_buffer_name.into(), vertex_buffer),
        (instance_buffer_name.into(), instance_buffer),
        (index_buffer_name.into(), index_buffer),
        (uniform_buffer_name.into(), uniform_buffer),
    ]);

    let textures = HashMap::new();

    let uniform = Binding {
        index: 0,
        shader_stages: ShaderStages::Vertex,
        info: UniformInfo {
            buffer: uniform_buffer_name.into(),
            size: uniform_size,
            offset: 0,
        },
    };
    let bind_group_name = "bind_group";
    let bind_group = BindGroup {
        uniforms: vec![uniform],
        ..Default::default()
    };
    let bind_groups = HashMap::from_iter([(bind_group_name.into(), bind_group)]);

    let attributes = vec![
        InputAttribute {
            item: InputItem::Float32x4,
            offset: 0,
            location: 0,
        },
        InputAttribute {
            item: InputItem::Float32x4,
            offset: VEC4_SIZE,
            location: 1,
        },
    ];
    let vertex_buffer = InputBuffer {
        buffer: vertex_buffer_name.into(),
        attributes,
        input_type: InputType::PerVertex,
        stride: VEC4_SIZE * 2,
    };

    let attributes = vec![
        InputAttribute {
            item: InputItem::Float32x4,
            offset: 0,
            location: 2,
        },
        InputAttribute {
            item: InputItem::Float32x4,
            offset: VEC4_SIZE,
            location: 3,
        },
        InputAttribute {
            item: InputItem::Float32x4,
            offset: VEC4_SIZE * 2,
            location: 4,
        },
        InputAttribute {
            item: InputItem::Float32x4,
            offset: VEC4_SIZE * 3,
            location: 5,
        },
    ];
    let instance_buffer = InputBuffer {
        buffer: instance_buffer_name.into(),
        attributes,
        input_type: InputType::PerInstance,
        stride: MAT4_SIZE,
    };

    let shader = Shader {
        path: "examples/shaders/color_mesh.wgsl".into(),
        vs_main: "vs_main".into(),
        fs_main: "fs_main".into(),
    };
    let pipeline_name = "pipeline";
    let pipeline = RenderPipeline {
        input_buffers: vec![vertex_buffer, instance_buffer],
        index_buffer: index_buffer_name.into(),
        bind_groups: vec![bind_group_name.into()],
        shader,
        depth: None,
    };
    let pipelines = HashMap::from_iter([(pipeline_name.into(), pipeline)]);

    let pass_name = "color_pass";
    let pass = Pass {
    	pipelines: vec![pipeline_name.into()],
    	color_attachments: vec![Attachment {
    		target: AttachmentTarget::Window,
    		size: Size2d { x: 800, y: 600 },
    		offset: Size2d::default(),
            clear: Some([0.0, 0.0, 0.0, 1.0]),
            store: true,
    	}],
        depth_attachment: None,
    };

    let passes = HashMap::from_iter([(pass_name.into(), pass)]);

    let stages = Stages(vec![PassGroup(vec![pass_name.into()])]);

    RenderModel {
        stages,
        passes,
        pipelines,
        bind_groups,
        textures,
        buffers,
    }
}

fn main() {}
