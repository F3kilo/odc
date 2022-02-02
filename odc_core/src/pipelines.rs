use crate::bind::BindGroups;
use crate::model as mdl;
use crate::res::Resources;
use std::collections::HashMap;
use std::fs;

pub struct Pipelines {
    render: HashMap<String, RenderPipeline>,
}

impl Pipelines {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(
        device: &wgpu::Device,
        model: &mdl::RenderModel,
        bind_groups: &BindGroups,
        window_format: wgpu::TextureFormat,
    ) -> Self {
        let factory = HandlesFactory { device, model, window_format };

        let render = model
            .pipelines
            .iter()
            .map(|(name, item)| {
                let pipeline = factory.create_pipeline(name, item, bind_groups);
                (name.clone(), pipeline)
            })
            .collect();

        Self { render }
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, pipeline: &str) {
    	let pipeline = &self.render[pipeline].0;
    	pass.set_pipeline(pipeline);
    }
}

struct RenderPipeline(wgpu::RenderPipeline);

impl RenderPipeline {
    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        Self(pipeline)
    }

    pub fn input_buffers_data(input_buffers: &[mdl::InputBuffer]) -> Vec<InputBufferLayout> {
        input_buffers.iter().map(InputBufferLayout::new).collect()
    }

    pub fn wgpu_vertex_buffer_layouts(data: &[InputBufferLayout]) -> Vec<wgpu::VertexBufferLayout> {
        data.iter()
            .map(|layout| wgpu::VertexBufferLayout {
                array_stride: layout.stride,
                step_mode: layout.step_mode,
                attributes: &layout.attributes,
            })
            .collect()
    }
}

struct InputBufferLayout {
    pub stride: u64,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

impl InputBufferLayout {
    pub fn new(input_buffer: &mdl::InputBuffer) -> Self {
        let step_mode = match input_buffer.input_type {
            mdl::InputType::PerVertex => wgpu::VertexStepMode::Vertex,
            mdl::InputType::PerInstance => wgpu::VertexStepMode::Instance,
        };

        let attributes = input_buffer
            .attributes
            .iter()
            .map(Self::wgpu_input_attributes)
            .collect();

        Self {
            stride: input_buffer.stride,
            step_mode,
            attributes,
        }
    }

    fn wgpu_input_attributes(attribute: &mdl::InputAttribute) -> wgpu::VertexAttribute {
        let format = match attribute.item {
            mdl::InputItem::Float16x2 => wgpu::VertexFormat::Float16x2,
            mdl::InputItem::Float16x4 => wgpu::VertexFormat::Float16x4,
            mdl::InputItem::Float32 => wgpu::VertexFormat::Float32,
            mdl::InputItem::Float32x2 => wgpu::VertexFormat::Float32x2,
            mdl::InputItem::Float32x3 => wgpu::VertexFormat::Float32x3,
            mdl::InputItem::Float32x4 => wgpu::VertexFormat::Float32x4,
            mdl::InputItem::Sint16x2 => wgpu::VertexFormat::Sint16x2,
            mdl::InputItem::Sint16x4 => wgpu::VertexFormat::Sint16x4,
            mdl::InputItem::Sint32 => wgpu::VertexFormat::Sint32,
            mdl::InputItem::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            mdl::InputItem::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            mdl::InputItem::Sint32x4 => wgpu::VertexFormat::Sint32x4,
            mdl::InputItem::Sint8x2 => wgpu::VertexFormat::Sint8x2,
            mdl::InputItem::Sint8x4 => wgpu::VertexFormat::Sint8x4,
            mdl::InputItem::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
            mdl::InputItem::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
            mdl::InputItem::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            mdl::InputItem::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            mdl::InputItem::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            mdl::InputItem::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            mdl::InputItem::Uint32 => wgpu::VertexFormat::Uint32,
            mdl::InputItem::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            mdl::InputItem::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            mdl::InputItem::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            mdl::InputItem::Uint8x2 => wgpu::VertexFormat::Uint8x2,
            mdl::InputItem::Uint8x4 => wgpu::VertexFormat::Uint8x4,
            mdl::InputItem::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            mdl::InputItem::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            mdl::InputItem::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            mdl::InputItem::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
        };

        wgpu::VertexAttribute {
            format,
            offset: attribute.offset,
            shader_location: attribute.location,
        }
    }
}

struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    model: &'a mdl::RenderModel,
    window_format: wgpu::TextureFormat,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_pipeline(
        &self,
        name: &str,
        info: &mdl::RenderPipeline,
        bind_groups: &BindGroups,
    ) -> RenderPipeline {
        use std::borrow::Cow;

        let shader_data = fs::read_to_string(&info.shader.path).expect("shader file not found");
        let shader_src = Cow::Owned(shader_data);
        let source = wgpu::ShaderSource::Wgsl(shader_src);
        let descriptor = wgpu::ShaderModuleDescriptor {
            label: None,
            source,
        };
        let shader_module = self.device.create_shader_module(&descriptor);

        let input_buffers_data = RenderPipeline::input_buffers_data(&info.input_buffers);
        let input_buffers = RenderPipeline::wgpu_vertex_buffer_layouts(&input_buffers_data);

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: &info.shader.vs_main,
            buffers: &input_buffers,
        };

        let bind_group_layouts: Vec<_> = info
            .bind_groups
            .iter()
            .map(|s| bind_groups.raw_layout(s))
            .collect();
        let layout = self.create_pipeline_layout(name, &bind_group_layouts);

        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Front),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let depth_stencil = info.depth.as_ref().map(|_| wgpu::DepthStencilState {
            format: Pipelines::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let color_targets = self.pipeline_targets(name);

        let fragment = Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: &info.shader.fs_main,
            targets: &color_targets,
        });

        let descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(name),
            layout: Some(&layout),
            vertex,
            primitive,
            depth_stencil,
            multisample: Default::default(),
            fragment,
            multiview: None,
        };

        RenderPipeline::new(self.device.create_render_pipeline(&descriptor))
    }

    pub fn create_pipeline_layout(
        &self,
        name: &str,
        layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        let descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some(name),
            bind_group_layouts: layouts,
            push_constant_ranges: &[],
        };

        self.device.create_pipeline_layout(&descriptor)
    }

    fn pipeline_targets(&self, pipeline: &str) -> Vec<wgpu::ColorTargetState> {
        for pass in self.model.passes.values() {
            if pass.pipelines.iter().any(|p| p == pipeline) {
                return pass
                    .color_attachments
                    .iter()
                    .map(|attachment| {
                        let format = match &attachment.target {
                        	mdl::AttachmentTarget::Window => self.window_format,
                        	mdl::AttachmentTarget::Texture(name) => {
                        		let texture_type = self.model.textures[name].typ;
                        		Resources::texture_format(texture_type)
                        	}
                        };
                        wgpu::ColorTargetState {
                            format,
                            blend: None,
                            write_mask: Default::default(),
                        }
                    })
                    .collect();
            }
        }
        panic!("pipeline {} is not used in any pass", pipeline);
    }
}
