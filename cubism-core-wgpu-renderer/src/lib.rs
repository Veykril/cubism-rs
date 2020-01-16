use wgpu::*;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

struct BoundTexture {
    bind_group: wgpu::BindGroup,
}

impl BoundTexture {
    pub fn new(
        texture: wgpu::Texture,
        sampler: &wgpu::Sampler,
        layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
    ) -> Self {
        let view = texture.create_default_view();

        // Create the texture bind group from the layout.
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout,
            bindings: &[
                Binding {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                Binding {
                    binding: 1,
                    resource: BindingResource::Sampler(sampler),
                },
            ],
        });

        BoundTexture { bind_group }
    }

    pub fn make_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        })
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[
                BindGroupLayoutBinding {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::SampledTexture {
                        multisampled: false,
                        dimension: TextureViewDimension::D2,
                    },
                },
                BindGroupLayoutBinding {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler,
                },
            ],
        })
    }
}

pub struct Renderer {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    uniform_bind_group: BindGroup,
    textures: Vec<BoundTexture>,
    texture_layout: BindGroupLayout,
    vertex_buffers: Vec<wgpu::Buffer>,
    index_buffers: Vec<(wgpu::Buffer, usize)>,
}

impl Renderer {
    /// Initializes a renderer.
    pub fn new(
        model: &cubism_core::Model,
        device: &Device,
        queue: &mut Queue,
        format: TextureFormat,
        textures: impl IntoIterator<Item = Texture>,
    ) -> Renderer {
        let vert = wgpu::read_spirv(std::io::Cursor::new(
            &include_bytes!("../shader/default.vert.spv")[..],
        ))
        .expect("vert");
        let frag = wgpu::read_spirv(std::io::Cursor::new(
            &include_bytes!("../shader/default.frag.spv")[..],
        ))
        .expect("frag");
        let vs_module = device.create_shader_module(&vert);
        let fs_module = device.create_shader_module(&frag);

        // Create the uniform matrix buffer.
        let uniform_buffer = device
            .create_buffer_mapped::<f32>(
                16,
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            )
            .fill_from_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]);
        let uniform_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            bindings: &[BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: BindingType::UniformBuffer { dynamic: false },
            }],
        });
        let uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &uniform_layout,
            bindings: &[Binding {
                binding: 0,
                resource: BindingResource::Buffer {
                    buffer: &uniform_buffer,
                    range: 0..64,
                },
            }],
        });

        let texture_layout = BoundTexture::layout(device);
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            bind_group_layouts: &[&uniform_layout, &texture_layout],
        });

        // Create the render pipeline.
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(RasterizationStateDescriptor {
                front_face: FrontFace::Cw,
                cull_mode: CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: PrimitiveTopology::TriangleList,
            color_states: &[ColorStateDescriptor {
                format,
                color_blend: BlendDescriptor {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
                },
                alpha_blend: BlendDescriptor {
                    src_factor: BlendFactor::OneMinusDstAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                write_mask: ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: IndexFormat::Uint16,
            vertex_buffers: &[
                // pos
                VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            shader_location: 0,
                            offset: 0,
                        },
                        VertexAttributeDescriptor {
                            format: VertexFormat::Float2,
                            shader_location: 1,
                            offset: 8,
                        },
                    ],
                },
            ],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mut vertex_buffers = Vec::with_capacity(model.drawable_count());
        let mut index_buffers = Vec::with_capacity(model.drawable_count());
        let mut temp = Vec::new();
        for cubism_core::Drawable {
            vertex_positions,
            vertex_uvs,
            indices,
            ..
        } in model.drawables()
        {
            temp.extend(
                vertex_positions
                    .iter()
                    .zip(vertex_uvs)
                    .map(|(&pos, &uv)| Vertex { pos, uv }),
            );
            vertex_buffers.push(
                device
                    .create_buffer_mapped(temp.len(), wgpu::BufferUsage::VERTEX)
                    .fill_from_slice(&temp),
            );
            index_buffers.push((
                device
                    .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
                    .fill_from_slice(indices),
                indices.len(),
            ));
            temp.clear();
        }

        let sampler = BoundTexture::make_sampler(&device);

        Renderer {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            textures: textures
                .into_iter()
                .map(|tex| BoundTexture::new(tex, &sampler, &texture_layout, &device))
                .collect(),
            texture_layout,
            vertex_buffers,
            index_buffers,
        }
    }

    /// Draws a model.
    pub fn draw_model(
        &mut self,
        device: &Device,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        model: &cubism_core::Model,
    ) {
        let mut drawables: Vec<_> = model.drawables().collect();
        drawables.sort_unstable_by_key(|d| d.render_order);

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.0,
                },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.uniform_bind_group, &[]);
        // pass by ref or value? Drawable is quite a big structure
        for drawable in &drawables {
            self.draw_drawable(device, &mut rpass, drawable).unwrap();
        }
    }

    fn update_buffers(
        &mut self,
        device: &Device,
        drawable: &cubism_core::Drawable,
    ) -> Result<(), ()> {
        let vtx_pos = drawable.vertex_positions;
        let vtx_uv = drawable.vertex_uvs;
        let vtx_buffer = vtx_pos
            .iter()
            .zip(vtx_uv)
            .map(|(&pos, &uv)| Vertex { pos, uv })
            .collect::<Vec<_>>();
        self.vertex_buffers[drawable.index] = device
            .create_buffer_mapped(vtx_buffer.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vtx_buffer);
        Ok(())
    }

    fn draw_drawable(
        &mut self,
        device: &Device,
        rpass: &mut RenderPass,
        drawable: &cubism_core::Drawable,
    ) -> Result<(), ()> {
        let dflags = drawable.dynamic_flags;
        if drawable.opacity <= 0.0 || !dflags.intersects(cubism_core::DynamicFlags::IS_VISIBLE) {
            return Ok(());
        }
        if dflags.intersects(cubism_core::DynamicFlags::VERTEX_POSITIONS_CHANGED) {
            self.update_buffers(device, drawable)?;
        }
        rpass.set_index_buffer(&self.index_buffers[drawable.index].0, 0);
        rpass.set_vertex_buffers(0, &[(&self.vertex_buffers[drawable.index], 0)]);
        rpass.set_bind_group(
            1,
            &self.textures[drawable.texture_index as usize].bind_group,
            &[],
        );
        rpass.draw_indexed(0..self.index_buffers[drawable.index].1 as u32, 0, 0..1);
        Ok(())
    }
}
