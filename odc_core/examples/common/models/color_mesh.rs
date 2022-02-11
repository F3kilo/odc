use super::{MAT4_SIZE, VEC4_SIZE};
use odc_core::mdl::*;

const UNIFORM_SIZE: u64 = MAT4_SIZE * 2;
const WINDOW_SIZE: Size2d = Size2d { x: 800, y: 600 };

pub fn color_mesh_model() -> RenderModel {
    let buffers = buffers();
    let textures = textures();
    let samplers = vec![];

    let bind_groups = bind_groups();
    let pipelines = pipelines();

    let passes = passes();

    RenderModel {
        passes,
        pipelines,
        bind_groups,
        textures,
        buffers,
        samplers,
    }
}

fn buffers() -> Buffers {
    Buffers {
        index: 2u64.pow(10),
        vertex: 2u64.pow(10),
        instance: 2u64.pow(16),
        uniform: UNIFORM_SIZE,
    }
}

fn textures() -> Vec<Texture> {
    let color_texture = Texture {
        typ: TextureType::Color {
            texel: TexelType::Unorm,
            texel_count: TexelCount::Four,
        },
        size: WINDOW_SIZE,
        window_source: true,
    };

    let depth_texture = Texture {
        typ: TextureType::Depth,
        size: WINDOW_SIZE,
        window_source: true,
    };

    vec![color_texture, depth_texture]
}

fn bind_groups() -> Vec<BindGroup> {
    let uniform = Binding {
        index: 0,
        shader_stages: ShaderStages::Vertex,
        info: UniformInfo {
            size: UNIFORM_SIZE,
            offset: 0,
        },
    };
    let bind_group = BindGroup {
        uniform: Some(uniform),
        ..Default::default()
    };

    vec![bind_group]
}

fn pipelines() -> Vec<RenderPipeline> {
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
    let vertex_buffer = InputInfo {
        attributes,
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
    let instance_buffer = InputInfo {
        attributes,
        stride: MAT4_SIZE,
    };

    let shader = Shader {
        path: "odc_core/examples/shaders/color_mesh.wgsl".into(),
        vs_main: "vs_main".into(),
        fs_main: "fs_main".into(),
    };

    let pipeline = RenderPipeline {
        input: Some(PipelineInpit {
            vertex: vertex_buffer,
            instance: instance_buffer,
        }),
        bind_groups: vec![0],
        shader,
        depth: Some(DepthOps {}),
    };

    vec![pipeline]
}

fn passes() -> Vec<Pass> {
    let pass = Pass {
        pipelines: vec![0],
        color_attachments: vec![Attachment {
            texture: 0,
            clear: Some([0.0, 0.0, 0.0, 1.0]),
            store: true,
        }],
        depth_attachment: Some(DepthAttachment { texture: 1 }),
    };

    vec![pass]
}
