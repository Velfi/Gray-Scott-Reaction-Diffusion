use bytemuck::{Pod, Zeroable};
use std::iter;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SimulationParams {
    feed_rate: f32,
    kill_rate: f32,
    delta_u: f32,
    delta_v: f32,
    width: u32,
    height: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct UVPair {
    u: f32,
    v: f32,
}

#[allow(dead_code)]
pub struct ReactionDiffusionSystem {
    pub width: usize,
    pub height: usize,
    feed_rate: f32,
    kill_rate: f32,
    delta_u: f32,
    delta_v: f32,
    uvs: Vec<UVPair>,

    // GPU resources
    device: wgpu::Device,
    queue: wgpu::Queue,
    uvs_buffers: [wgpu::Buffer; 2], // Double buffering
    current_buffer: usize,
    params_buffer: wgpu::Buffer,
    bind_groups: [wgpu::BindGroup; 2], // Double buffering
    compute_pipeline: wgpu::ComputePipeline,
}

impl ReactionDiffusionSystem {
    pub async fn new(
        width: usize,
        height: usize,
        feed_rate: f32,
        kill_rate: f32,
        delta_u: f32,
        delta_v: f32,
    ) -> Self {
        assert!(
            width <= isize::MAX as usize,
            "Reaction diffusion system width must be less than or equal to {} but {} was passed",
            isize::MAX,
            width
        );
        assert!(
            height <= isize::MAX as usize,
            "Reaction diffusion system height must be less than or equal to {} but {} was passed",
            isize::MAX,
            height
        );

        let vec_capacity = width * height;
        let uvs: Vec<UVPair> = iter::repeat(UVPair { u: 1.0, v: 0.0 })
            .take(vec_capacity)
            .collect();

        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
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

        // Create double buffers
        let uvs_buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UVs Buffer 0"),
                size: (vec_capacity * std::mem::size_of::<UVPair>()) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: true,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("UVs Buffer 1"),
                size: (vec_capacity * std::mem::size_of::<UVPair>()) as u64,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: true,
            }),
        ];

        // Write initial UVs data to both buffers
        for buffer in &uvs_buffers {
            let slice = buffer.slice(..);
            slice
                .get_mapped_range_mut()
                .copy_from_slice(bytemuck::cast_slice(&uvs));
            buffer.unmap();
        }

        let params = SimulationParams {
            feed_rate,
            kill_rate,
            delta_u,
            delta_v,
            width: width as u32,
            height: height as u32,
        };

        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bind group layout and pipeline
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/reaction_diffusion.wgsl").into(),
            ),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        // Create bind groups for both buffers (input/output swapped)
        let bind_groups = [
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group 0"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uvs_buffers[0].as_entire_binding(), // input
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uvs_buffers[1].as_entire_binding(), // output
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            }),
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group 1"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uvs_buffers[1].as_entire_binding(), // input
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: uvs_buffers[0].as_entire_binding(), // output
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: params_buffer.as_entire_binding(),
                    },
                ],
            }),
        ];

        Self {
            width,
            height,
            feed_rate,
            kill_rate,
            delta_u,
            delta_v,
            uvs,
            device,
            queue,
            uvs_buffers,
            current_buffer: 0,
            params_buffer,
            bind_groups,
            compute_pipeline,
        }
    }

    pub fn uvs(&mut self) -> &[(f32, f32)] {
        // Only read back when needed
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (self.width * self.height * std::mem::size_of::<UVPair>()) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Readback Encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &self.uvs_buffers[1 - self.current_buffer],
            0,
            &staging_buffer,
            0,
            staging_buffer.size(),
        );
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        self.uvs = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging_buffer.unmap();

        unsafe { std::mem::transmute(self.uvs.as_slice()) }
    }

    pub fn set(&mut self, x: isize, y: isize, v: (f32, f32)) {
        let index = self.get_index(x, y);
        let v = (v.0.clamp(-1.0, 1.0), v.1.clamp(-1.0, 1.0));

        // Update CPU-side data
        self.uvs[index] = UVPair { u: v.0, v: v.1 };

        // Update GPU buffer
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: std::mem::size_of::<UVPair>() as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        // Write the new value
        let slice = staging_buffer.slice(..);
        let mut view = slice.get_mapped_range_mut();
        view.copy_from_slice(bytemuck::cast_slice(&[UVPair { u: v.0, v: v.1 }]));
        drop(view);
        staging_buffer.unmap();

        // Copy to the main buffer
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Set Value Encoder"),
            });
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uvs_buffers[self.current_buffer],
            (index * std::mem::size_of::<UVPair>()) as u64,
            std::mem::size_of::<UVPair>() as u64,
        );
        self.queue.submit(Some(encoder.finish()));
    }

    pub fn set_all(&mut self, values: &[(f32, f32)]) {
        assert_eq!(
            values.len(),
            self.width * self.height,
            "Values length must match grid size"
        );

        // Update CPU-side data
        for (i, (u, v)) in values.iter().enumerate() {
            self.uvs[i] = UVPair {
                u: u.clamp(-1.0, 1.0),
                v: v.clamp(-1.0, 1.0),
            };
        }

        // Update GPU buffer
        let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: (self.width * self.height * std::mem::size_of::<UVPair>()) as u64,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: true,
        });

        // Write all values
        let slice = staging_buffer.slice(..);
        let mut view = slice.get_mapped_range_mut();
        view.copy_from_slice(bytemuck::cast_slice(&self.uvs));
        drop(view);
        staging_buffer.unmap();

        // Copy to the main buffer
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Set All Values Encoder"),
            });
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.uvs_buffers[self.current_buffer],
            0,
            staging_buffer.size(),
        );
        self.queue.submit(Some(encoder.finish()));
    }

    fn get_index(&self, x: isize, y: isize) -> usize {
        let width = self.width as isize;
        let height = self.height as isize;
        let x = (x + width) % width;
        let y = (y + height) % height;
        (y * width + x) as usize
    }

    pub fn update(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
            });

            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.bind_groups[self.current_buffer], &[]);
            compute_pass.dispatch_workgroups(
                (self.width as u32 + 7) / 8,
                (self.height as u32 + 7) / 8,
                1,
            );
        }

        self.queue.submit(Some(encoder.finish()));
        self.current_buffer = 1 - self.current_buffer; // Toggle between 0 and 1
    }

    pub fn update_rates(&mut self, feed_rate: f32, kill_rate: f32) {
        self.feed_rate = feed_rate;
        self.kill_rate = kill_rate;

        // Update the params buffer with new rates
        let params = SimulationParams {
            feed_rate,
            kill_rate,
            delta_u: self.delta_u,
            delta_v: self.delta_v,
            width: self.width as u32,
            height: self.height as u32,
        };

        let staging_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Staging Buffer"),
                contents: bytemuck::cast_slice(&[params]),
                usage: wgpu::BufferUsages::COPY_SRC,
            });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Update Rates Encoder"),
            });
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.params_buffer,
            0,
            std::mem::size_of::<SimulationParams>() as u64,
        );
        self.queue.submit(Some(encoder.finish()));

        // Recreate the bind group with the updated params buffer
        let bind_group_layout =
            self.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });

        self.bind_groups[self.current_buffer] =
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uvs_buffers[self.current_buffer].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.uvs_buffers[1 - self.current_buffer].as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.params_buffer.as_entire_binding(),
                    },
                ],
            });
    }
}
