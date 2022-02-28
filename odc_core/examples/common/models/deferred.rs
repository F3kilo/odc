use super::{MAT4_SIZE, VEC4_SIZE};
use odc_core::mdl::*;

const UNIFORM_SIZE: u64 = MAT4_SIZE * 2;
const WINDOW_SIZE: Size2d = Size2d { x: 800, y: 600 };

pub fn deferred_model() -> RenderModel {
    let buffers = buffers();
    let textures = textures();
    let samplers = samplers();

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
    let position = Texture {
        typ: TextureType::Color {
            texel: TexelType::Float(BytesPerFloatTexel::Four),
            texel_count: TexelCount::Four,
        },
        size: WINDOW_SIZE.into(),
        mip_levels: 1,
        sample_count: 1,
        window_source: true,
        writable: false,
    };

    let albedo = Texture {
        typ: TextureType::Color {
            texel: TexelType::Unorm,
            texel_count: TexelCount::Four,
        },
        size: WINDOW_SIZE.into(),
        mip_levels: 1,
        sample_count: 1,
        window_source: true,
        writable: false,
    };

    let light = Texture {
        typ: TextureType::Color {
            texel: TexelType::Unorm,
            texel_count: TexelCount::Four,
        },
        size: WINDOW_SIZE.into(),
        mip_levels: 1,
        sample_count: 1,
        window_source: true,
        writable: false,
    };

    let final_tex = Texture {
        typ: TextureType::Color {
            texel: TexelType::Unorm,
            texel_count: TexelCount::Four,
        },
        size: WINDOW_SIZE.into(),
        mip_levels: 1,
        sample_count: 1,
        window_source: true,
        writable: false,
    };

    let depth = Texture {
        typ: TextureType::Depth,
        size: WINDOW_SIZE.into(),
        mip_levels: 1,
        sample_count: 1,
        window_source: true,
        writable: false,
    };

    vec![position, albedo, light, depth, final_tex]
}

fn samplers() -> Vec<Sampler> {
    let position_sampler = Sampler::NonFilter;
    vec![position_sampler]
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

    let deferred_group = BindGroup {
        uniform: Some(uniform),
        ..Default::default()
    };

    let position_texture = Binding {
        index: 0,
        shader_stages: ShaderStages::Fragment,
        info: TextureInfo {
            texture: 0,
            dimension: TextureViewDimension::D2,
        },
    };

    let position_sampler = Binding {
        index: 2,
        shader_stages: ShaderStages::Fragment,
        info: SamplerInfo { sampler: 0 },
    };

    let light_group = BindGroup {
        uniform: None,
        textures: vec![position_texture],
        samplers: vec![position_sampler],
    };

    let albedo_texture = Binding {
        index: 0,
        shader_stages: ShaderStages::Fragment,
        info: TextureInfo {
            texture: 1,
            dimension: TextureViewDimension::D2,
        },
    };

    let light_texture = Binding {
        index: 1,
        shader_stages: ShaderStages::Fragment,
        info: TextureInfo {
            texture: 2,
            dimension: TextureViewDimension::D2,
        },
    };

    let albedo_light_sampler = Binding {
        index: 2,
        shader_stages: ShaderStages::Fragment,
        info: SamplerInfo { sampler: 0 },
    };

    let final_group = BindGroup {
        uniform: None,
        textures: vec![albedo_texture, light_texture],
        samplers: vec![albedo_light_sampler],
    };

    vec![deferred_group, light_group, final_group]
}

fn position_pipeline() -> RenderPipeline {
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
        path: "odc_core/examples/shaders/deferred.wgsl".into(),
        vs_main: "vs_main".into(),
        fs_main: "fs_main".into(),
    };

    RenderPipeline {
        input: Some(PipelineInpit {
            vertex: vertex_buffer,
            instance: instance_buffer,
        }),
        bind_groups: vec![0],
        shader,
        blend: vec![None, None],
        depth: Some(DepthOps {}),
    }
}

fn light_pipeline() -> RenderPipeline {
    let attributes = vec![InputAttribute {
        item: InputItem::Float32x4,
        offset: 0,
        location: 0,
    }];

    let vertex_buffer = InputInfo {
        attributes, // TODO: Without it wgpu will skip this buffer and vertex buffer will be used instead instance buffer.
        stride: 0,
    };

    let attributes = vec![InputAttribute {
        item: InputItem::Float32x4,
        offset: 0,
        location: 1,
    }];

    let instance_buffer = InputInfo {
        attributes,
        stride: VEC4_SIZE,
    };

    let shader = Shader {
        path: "odc_core/examples/shaders/deferred_light.wgsl".into(),
        vs_main: "vs_main".into(),
        fs_main: "fs_main".into(),
    };

    let blend = BlendState {
        color: BlendComponent {
            dst_factor: BlendFactor::One,
            src_factor: BlendFactor::One,
            operation: BlendOperation::Add,
        },
        alpha: BlendComponent::REPLACE,
    };

    RenderPipeline {
        input: Some(PipelineInpit {
            vertex: vertex_buffer,
            instance: instance_buffer,
        }),
        bind_groups: vec![1],
        shader,
        blend: vec![Some(blend)],
        depth: None,
    }
}

fn final_pipeline() -> RenderPipeline {
    let shader = Shader {
        path: "odc_core/examples/shaders/deferred_final.wgsl".into(),
        vs_main: "vs_main".into(),
        fs_main: "fs_main".into(),
    };

    RenderPipeline {
        input: None,
        bind_groups: vec![2],
        shader,
        blend: vec![None],
        depth: None,
    }
}

fn pipelines() -> Vec<RenderPipeline> {
    vec![position_pipeline(), light_pipeline(), final_pipeline()]
}

fn passes() -> Vec<Pass> {
    let position_attachment = Attachment {
        texture: 0,
        clear: Some([0.0, 0.0, 0.0, 0.0]),
        store: true,
    };

    let albedo_attachment = Attachment {
        texture: 1,
        clear: Some([0.0, 0.0, 0.0, 0.0]),
        store: true,
    };

    let deferred = Pass {
        pipelines: vec![0],
        color_attachments: vec![position_attachment, albedo_attachment],
        depth_attachment: Some(DepthAttachment { texture: 3 }),
    };

    let light = Pass {
        pipelines: vec![1],
        color_attachments: vec![Attachment {
            texture: 2,
            clear: Some([0.0, 0.0, 0.0, 0.0]),
            store: true,
        }],
        depth_attachment: None,
    };

    let final_pass = Pass {
        pipelines: vec![2],
        color_attachments: vec![Attachment {
            texture: 4,
            clear: Some([0.0, 0.0, 0.0, 1.0]),
            store: true,
        }],
        depth_attachment: None,
    };

    vec![deferred, light, final_pass]
}
