use instant::Instant;
use std::borrow::Cow;
#[allow(unused_imports)]
use tracing::{error, info, warn};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::util::DeviceExt;
use winit::{event::*, event_loop::EventLoop, window::Window};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

const VERTICES: &[Vertex] = &[
    // Triangle 1
    Vertex {
        position: [-0.8, -0.8],
    },
    Vertex {
        position: [0.8, -0.8],
    },
    Vertex {
        position: [0.8, 0.8],
    },
    // Triangle 2
    Vertex {
        position: [-0.8, -0.8],
    },
    Vertex {
        position: [0.8, 0.8],
    },
    Vertex {
        position: [-0.8, 0.8],
    },
];
const NUM_VERTICES: u32 = VERTICES.len() as u32;
const NUM_SQUARE_INSTANCES: u32 = (GRID_SIZE * GRID_SIZE) as u32;

const GRID_SIZE: usize = 64;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GridSizeUniform {
    grid: [f32; 2],
}

impl GridSizeUniform {
    fn new() -> Self {
        Self {
            grid: [GRID_SIZE as f32, GRID_SIZE as f32],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CellState {
    state_a: [u32; GRID_SIZE * GRID_SIZE],
    state_b: [u32; GRID_SIZE * GRID_SIZE],
}

impl CellState {
    fn new() -> Self {
        use rand::Rng;

        let mut grid_a = [0; GRID_SIZE * GRID_SIZE];
        let mut rng = rand::thread_rng();

        for cell in grid_a.iter_mut().take(GRID_SIZE * GRID_SIZE) {
            *cell = rng.gen_range(0..=1);
        }

        let grid_b = [0; GRID_SIZE * GRID_SIZE];

        Self {
            state_a: grid_a,
            state_b: grid_b,
        }
    }
}

// We use two bind groups to enable the ping pong buffer pattern
struct BindGroups {
    group_a: wgpu::BindGroup,
    group_b: wgpu::BindGroup,
}

struct State<'a> {
    window: &'a Window,
    window_size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    vertex_buffer: wgpu::Buffer,
    bind_groups: BindGroups,
    step: u32, // how many simulation steps have been run
    render_pipeline: wgpu::RenderPipeline,
    compute_pipeline: wgpu::ComputePipeline,
    cell_state_storage_buffer_state_a: wgpu::Buffer,
    cell_state_storage_buffer_state_b: wgpu::Buffer,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> Self {
        let mut window_size = window.inner_size();
        window_size.width = window_size.width.max(1);
        window_size.height = window_size.height.max(1);

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .expect("Failed to create device");

        let config = surface
            .get_default_config(&adapter, window_size.width, window_size.height)
            .unwrap();
        surface.configure(&device, &config);

        // Create the vertex buffer
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // Create grid size uniform buffer
        let grid_size_uniform = GridSizeUniform::new();
        let grid_size_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Grid Uniforms"),
                contents: bytemuck::cast_slice(&[grid_size_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        // Create cell state storage buffer
        let cell_state = CellState::new();
        let cell_state_storage_buffer_state_a =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cell State Storage Buffer A"),
                contents: bytemuck::cast_slice(&[cell_state.state_a]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let cell_state_storage_buffer_state_b =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Cell State Storage Buffer B"),
                contents: bytemuck::cast_slice(&[cell_state.state_b]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::from_iter(
                        wgpu::ShaderStages::VERTEX
                            | wgpu::ShaderStages::FRAGMENT
                            | wgpu::ShaderStages::COMPUTE,
                    ),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // cell state input buffer (read only)
                    binding: 1,
                    visibility: wgpu::ShaderStages::from_iter(
                        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
                    ),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // cell state output buffer (read-write)
                    binding: 2,
                    visibility: wgpu::ShaderStages::from_iter(wgpu::ShaderStages::COMPUTE),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("bind_group_layout"),
        });

        let bind_groups = BindGroups {
            group_a: device.create_bind_group(&wgpu::BindGroupDescriptor {
                // bind group a:
                // - cell state input: state a
                // - cell state output: state b
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: grid_size_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: cell_state_storage_buffer_state_a.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cell_state_storage_buffer_state_b.as_entire_binding(),
                    },
                ],
                label: Some("Cell Renderer Bind Group A"),
            }),
            group_b: device.create_bind_group(&wgpu::BindGroupDescriptor {
                // bind group b:
                // - cell state input: state b
                // - cell state output: state a
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: grid_size_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: cell_state_storage_buffer_state_b.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cell_state_storage_buffer_state_a.as_entire_binding(),
                    },
                ],
                label: Some("Cell Renderer Bind Group B"),
            }),
        };

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create compute pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "cs_main",
        });

        Self {
            window,
            window_size,
            surface,
            device,
            queue,
            config,
            vertex_buffer,
            bind_groups,
            step: 0,
            render_pipeline,
            compute_pipeline,
            cell_state_storage_buffer_state_a,
            cell_state_storage_buffer_state_b,
        }
    }

    fn resize(&mut self, new_window_size: winit::dpi::PhysicalSize<u32>) {
        // winit will panic if the window is 0x0
        if new_window_size.width > 0 && new_window_size.height > 0 {
            // Reconfigure the surface with the new size
            self.window_size = new_window_size;
            self.config.width = new_window_size.width;
            self.config.height = new_window_size.height;
            self.surface.configure(&self.device, &self.config);
            // On macos the window needs to be redrawn manually after resizing
            self.window.request_redraw();
        }
    }

    #[allow(dead_code, unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        // if we need to update state, we can do so here
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // In general, You want to do the compute pass before the render pass because it allows
        // the render pass to immediately use the latest results from the compute pass.

        // Compute Pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);

            let bind_group = if self.step % 2 == 0 {
                &self.bind_groups.group_a
            } else {
                &self.bind_groups.group_b
            };

            compute_pass.set_bind_group(0, bind_group, &[]);
            const WORKGROUP_SIZE: usize = 8;
            let workgroup_count = (GRID_SIZE as f32 / WORKGROUP_SIZE as f32).ceil() as u32;

            compute_pass.dispatch_workgroups(workgroup_count, workgroup_count, 1);
        }

        // We increment the step count between the compute pass and render pass so that the
        // output buffer of the compute pipeline becomes the input buffer for the render pipeline.
        self.step += 1;

        // Render Pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 32.0 / 255.0,
                            g: 32.0 / 255.0,
                            b: 32.0 / 255.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let bind_group = if self.step % 2 == 0 {
                &self.bind_groups.group_a
            } else {
                &self.bind_groups.group_b
            };

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..NUM_VERTICES, 0..NUM_SQUARE_INSTANCES);
        }

        // submit command buffers for execution
        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    fn randomise_grid(&mut self) {
        // Reset the step counter
        self.step = 0;

        // Initialise new cell state
        let new_cell_state = CellState::new();

        // Write the new cell states into the buffers
        self.queue.write_buffer(
            &self.cell_state_storage_buffer_state_a,
            0,
            bytemuck::cast_slice(&[new_cell_state.state_a]),
        );
        self.queue.write_buffer(
            &self.cell_state_storage_buffer_state_b,
            0,
            bytemuck::cast_slice(&[new_cell_state.state_b]),
        );
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    // Set up logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_error_panic_hook::set_once();
            tracing_wasm::set_as_global_default();
        } else {
            tracing_subscriber::fmt::init()
        }
    }
    // Logging usage
    // info!("debug: info log message");
    // warn!("debug: warning log message");
    // error!("debug: error log message");

    let event_loop = EventLoop::new().unwrap();

    #[allow(unused_mut)]
    let mut builder = winit::window::WindowBuilder::new();

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("life-wgpu")
            .expect("Failed to get canvas with id 'life-wgpu'")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Failed to get canvas");
        builder = builder.with_canvas(Some(canvas));
    }
    let window = builder.build(&event_loop).unwrap();
    window.set_title("Life wgpu");

    let mut state = State::new(&window).await;

    let mut last_update_time = Instant::now();

    event_loop
        .run(move |event, target| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key:
                                        winit::keyboard::PhysicalKey::Code(
                                            winit::keyboard::KeyCode::Escape,
                                        ),
                                    ..
                                },
                            ..
                        } => target.exit(),
                        // Capuring input this way works for both native and web.
                        // However, for web, the canvas must be focused for the input to be captured.
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key:
                                        winit::keyboard::PhysicalKey::Code(
                                            winit::keyboard::KeyCode::KeyR,
                                        ),
                                    ..
                                },
                            ..
                        } => {
                            state.randomise_grid();
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.window_size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                const UPDATE_INTERVAL: u128 = 20; // in milliseconds

                let now = Instant::now();
                if now.duration_since(last_update_time).as_millis() >= UPDATE_INTERVAL {
                    // Draw the next frame of the simulation
                    state.window.request_redraw();

                    last_update_time = now;
                }

                // When the event loop finishes, immediately begin a new iteration.
                // This is needed to prevent the event loop from idling.
                target.set_control_flow(winit::event_loop::ControlFlow::Poll);
            }
            _ => {}
        })
        .expect("Failed to run event loop");
}
