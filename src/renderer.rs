use crate::lut_manager::LutData;
use bytemuck::{Pod, Zeroable};
use fontdue::Font;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    window_aspect_ratio: f32,
    simulation_aspect_ratio: f32,
    is_lut_reversed: u32,
}

pub struct Renderer {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    diffusion_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    diffusion_texture: wgpu::Texture,
    diffusion_bind_group: wgpu::BindGroup,
    simulation_texture_width: u32,
    simulation_texture_height: u32,
    lut_buffer: wgpu::Buffer,
    text_texture: Option<wgpu::Texture>,
    text_bind_group: Option<wgpu::BindGroup>,
    text_bind_group_layout: wgpu::BindGroupLayout,
    text_sampler: wgpu::Sampler,
    text_size: f32,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window, width: u32, height: u32) -> Self {
        let size = window.inner_size();

        // Calculate text size as 1/40th of window height
        let text_size = size.height as f32 / 40.0;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let uniforms = Uniforms {
            window_aspect_ratio: size.width as f32 / size.height as f32,
            simulation_aspect_ratio: width as f32 / height as f32,
            is_lut_reversed: 0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let lut_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("LUT Buffer"),
            size: 768 * std::mem::size_of::<u32>() as u64, // 256 * 3 (RGB) values
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let diffusion_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Diffusion Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let diffusion_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Diffusion Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let text_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Text Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Render Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render.wgsl").into()),
        });

        let diffusion_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Diffusion Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let text_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Text Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let diffusion_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Diffusion Bind Group"),
            layout: &diffusion_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &diffusion_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&diffusion_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: lut_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&diffusion_bind_group_layout],
            push_constant_ranges: &[],
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&text_bind_group_layout],
            push_constant_ranges: &[],
        });

        let diffusion_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Diffusion Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Text Pipeline"),
            layout: Some(&text_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_text_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            diffusion_pipeline,
            text_pipeline,
            uniforms,
            uniform_buffer,
            diffusion_texture,
            diffusion_bind_group,
            simulation_texture_width: width,
            simulation_texture_height: height,
            lut_buffer,
            text_texture: None,
            text_bind_group: None,
            text_bind_group_layout,
            text_sampler,
            text_size,
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update window aspect ratio while keeping simulation aspect ratio
            self.uniforms.window_aspect_ratio = new_size.width as f32 / new_size.height as f32;

            // Update text size based on new window height
            self.text_size = new_size.height as f32 / 40.0;

            // Update uniform buffer
            self.queue.write_buffer(
                &self.uniform_buffer,
                0,
                bytemuck::cast_slice(&[self.uniforms]),
            );
        }
    }

    pub fn update_texture(&mut self, uvs: &[(f32, f32)]) {
        let data: Vec<f32> = uvs.iter().flat_map(|&(u, v)| [u, v]).collect();
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.diffusion_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(self.simulation_texture_width * 8), // 2 f32s per pixel
                rows_per_image: Some(self.simulation_texture_height),
            },
            wgpu::Extent3d {
                width: self.simulation_texture_width,
                height: self.simulation_texture_height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn update_lut(&mut self, lut_data: &LutData) {
        let mut lut_buffer_data = vec![0u32; 768]; // 256 * 3 for RGB

        // Copy RGB values directly
        for i in 0..256 {
            lut_buffer_data[i] = lut_data.red[i] as u32;
            lut_buffer_data[i + 256] = lut_data.green[i] as u32;
            lut_buffer_data[i + 512] = lut_data.blue[i] as u32;
        }

        self.queue
            .write_buffer(&self.lut_buffer, 0, bytemuck::cast_slice(&lut_buffer_data));
    }

    pub fn render_text(&mut self, text: &str, font: &Font, window_size: PhysicalSize<u32>) {
        // Calculate text layout
        let line_height = self.text_size * 1.5;
        let lines: Vec<&str> = text.lines().collect();

        // Calculate total height of text block
        let total_height = lines.len() as f32 * line_height;

        // Add padding around text
        let padding = self.text_size * 0.5; // Padding is half the text size

        // Calculate the maximum width of all lines for the background box
        let mut max_width = 0.0;
        for line in lines.iter() {
            let mut line_width = 0.0;
            for ch in line.chars() {
                let metrics = font.metrics(ch, self.text_size);
                line_width += metrics.advance_width;
            }
            max_width = f32::max(max_width, line_width);
        }

        // Calculate starting positions with padding
        let start_x = padding;
        let start_y = (window_size.height as f32 - total_height) / 2.0;

        // Create a texture to hold the text
        let texture_width = window_size.width;
        let texture_height = window_size.height;

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Text Texture"),
            size: wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Create a buffer to hold the text bitmap (4 bytes per pixel for RGBA)
        let mut text_bitmap = vec![0u8; (texture_width * texture_height * 4) as usize];

        // Draw the background box with padding
        let box_left = (start_x - padding) as usize;
        let box_top = (start_y - padding) as usize;
        let box_width = (max_width + padding * 2.0) as usize;
        let box_height = (total_height + padding * 2.0) as usize;

        // Fill the background box area with a solid black background
        for y in box_top..box_top + box_height {
            for x in box_left..box_left + box_width {
                if y < texture_height as usize && x < texture_width as usize {
                    let idx = (y * texture_width as usize + x) * 4;
                    if idx + 3 < text_bitmap.len() {
                        // RGBA: (0,0,0,255) for solid black
                        text_bitmap[idx] = 0; // R = 0
                        text_bitmap[idx + 1] = 0; // G = 0
                        text_bitmap[idx + 2] = 0; // B = 0
                        text_bitmap[idx + 3] = 255; // A = 255 (fully opaque)
                    }
                }
            }
        }

        // Render each line of text
        for (i, line) in lines.iter().enumerate() {
            let baseline_y = start_y + i as f32 * line_height + self.text_size;
            let mut x_position = start_x;

            for ch in line.chars() {
                let (raster_metrics, bitmap) = font.rasterize(ch, self.text_size);
                let font_metrics = font.metrics(ch, self.text_size);

                let glyph_y = baseline_y + font_metrics.bounds.ymin;

                // Copy bitmap to our texture data
                for y in 0..raster_metrics.height {
                    for x in 0..raster_metrics.width {
                        let src_y = raster_metrics.height - 1 - y;
                        let src_idx = src_y * raster_metrics.width + x;

                        let dst_x = x_position as usize + x;
                        let dst_y = glyph_y as usize + y;

                        if dst_y < texture_height as usize && dst_x < texture_width as usize {
                            let dst_idx = (dst_y * texture_width as usize + dst_x) * 4;
                            if dst_idx + 3 < text_bitmap.len()
                                && src_idx < bitmap.len()
                                && bitmap[src_idx] > 0
                            {
                                // White text (255,255,255) with alpha from the font rasterizer
                                text_bitmap[dst_idx] = 255; // R = 255
                                text_bitmap[dst_idx + 1] = 255; // G = 255
                                text_bitmap[dst_idx + 2] = 255; // B = 255
                                text_bitmap[dst_idx + 3] = bitmap[src_idx]; // A from font
                            }
                        }
                    }
                }

                x_position += font_metrics.advance_width;
            }
        }

        // Upload texture data
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &text_bitmap,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(texture_width * 4), // 4 bytes per pixel for RGBA
                rows_per_image: Some(texture_height),
            },
            wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
        );

        // Create bind group for text texture
        let text_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Text Bind Group"),
            layout: &self.text_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.text_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.lut_buffer.as_entire_binding(),
                },
            ],
        });

        self.text_texture = Some(texture);
        self.text_bind_group = Some(text_bind_group);
    }

    pub fn clear_text(&mut self) {
        self.text_texture = None;
        self.text_bind_group = None;
    }

    pub fn render(&mut self, view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // Render the main simulation
            render_pass.set_pipeline(&self.diffusion_pipeline);
            render_pass.set_bind_group(0, &self.diffusion_bind_group, &[]);
            render_pass.draw(0..4, 0..1);

            // Render text if available and text bind group exists
            if let Some(text_bind_group) = &self.text_bind_group {
                render_pass.set_pipeline(&self.text_pipeline);
                render_pass.set_bind_group(0, text_bind_group, &[]);
                render_pass.draw(0..4, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }

    pub fn set_lut_reversed(&mut self, reversed: bool) {
        self.uniforms.is_lut_reversed = if reversed { 1 } else { 0 };
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    pub fn is_lut_reversed(&self) -> bool {
        self.uniforms.is_lut_reversed == 1
    }
}
