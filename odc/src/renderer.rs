use crate::{GfxDevice, MeshBuffers, RenderInfo, StaticMesh, Vertex};
use std::borrow::Cow;
use std::mem;
use std::num::NonZeroU64;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferSize, BufferUsages, Color,
    CommandBuffer, CommandEncoder, FragmentState, LoadOp, Operations, PipelineLayout,
    PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureFormat, TextureView, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

pub struct BasicRenderer {
    uniform_buffer: Buffer,
    uniform_binding: BindGroup,
    pipeline: RenderPipeline,
}

impl BasicRenderer {
    pub fn new(
        device: &GfxDevice,
        output_format: TextureFormat,
        instances_binding_layout: &BindGroupLayout,
    ) -> Self {
        let uniform_size = Self::uniform_size();
        let uniform_buffer = crate::create_gpu_buffer(
            device,
            uniform_size.get(),
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        );

        let uniform_binding_layout = Self::create_bind_group_layout(device, uniform_size);
        let uniform_binding =
            Self::create_uniform_binding(device, &uniform_buffer, &uniform_binding_layout);

        let pipeline_layout =
            Self::create_pipeline_layout(device, instances_binding_layout, &uniform_binding_layout);
        let pipeline = Self::create_pipeline(device, &pipeline_layout, output_format);

        Self {
            pipeline,
            uniform_binding,
            uniform_buffer,
        }
    }

    pub fn update(&self, device: &GfxDevice, info: &RenderInfo) {
        let render_data = bytemuck::bytes_of(info);
        device
            .queue
            .write_buffer(&self.uniform_buffer, 0, render_data);
    }

    pub fn render<'a>(
        &self,
        mut encoder: CommandEncoder,
        mesh_buffers: &MeshBuffers,
        instances_binding: &BindGroup,
        view: &TextureView,
        draws: impl Iterator<Item = &'a StaticMesh>,
    ) -> CommandBuffer {
        let attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
        };
        let attachments = [attachment];
        let render_pass_descriptor = RenderPassDescriptor {
            color_attachments: &attachments,
            ..Default::default()
        };

        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.pipeline);
            mesh_buffers.bind(&mut render_pass);
            render_pass.set_bind_group(0, instances_binding, &[]);
            render_pass.set_bind_group(1, &self.uniform_binding, &[]);
            for draw in draws {
                render_pass.draw_indexed(
                    draw.indices.clone(),
                    draw.base_vertex,
                    draw.instances.clone(),
                );
            }
        }
        encoder.finish()
    }

    fn create_bind_group_layout(device: &GfxDevice, uniform_size: BufferSize) -> BindGroupLayout {
        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(uniform_size),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[uniform_entry],
        };
        device.device.create_bind_group_layout(&descriptor)
    }

    fn create_uniform_binding(
        device: &GfxDevice,
        uniform: &Buffer,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = [BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
    }

    fn create_shader(device: &GfxDevice) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("shader.wgsl"));
        let source = ShaderSource::Wgsl(shader_src);
        let descriptor = ShaderModuleDescriptor {
            label: None,
            source,
        };
        device.device.create_shader_module(&descriptor)
    }

    fn create_pipeline_layout(
        device: &GfxDevice,
        instances_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> PipelineLayout {
        let layouts = [instances_layout, uniform_layout];
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };
        device.device.create_pipeline_layout(&descriptor)
    }

    fn create_pipeline(
        device: &GfxDevice,
        layout: &PipelineLayout,
        output_format: TextureFormat,
    ) -> RenderPipeline {
        let attributes = [
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: Vertex::position_offset() as _,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: Vertex::color_offset() as _,
                shader_location: 1,
            },
        ];

        let vertex_layout = VertexBufferLayout {
            array_stride: Vertex::size() as _,
            attributes: &attributes,
            step_mode: VertexStepMode::Vertex,
        };

        let shader = Self::create_shader(device);

        let vertex = VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_layout],
        };

        let formats = [output_format.into()];
        let fragment = Some(FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &formats,
        });

        let descriptor = RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex,
            fragment,
            primitive: Default::default(),
            multisample: Default::default(),
            depth_stencil: None,
            multiview: None,
        };

        device.device.create_render_pipeline(&descriptor)
    }

    fn uniform_size() -> BufferSize {
        NonZeroU64::new(mem::size_of::<RenderInfo>() as _).expect("Zero sized uniform")
    }
}
