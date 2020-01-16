use std::{
    fs::File,
    io::Cursor,
    iter::FromIterator,
    path::{Path, PathBuf},
    time::Instant,
};
use winit::event_loop::EventLoop;

const SWAPCHAIN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

fn load_texture(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    path: &Path,
) -> wgpu::Texture {
    let image = image::load(Cursor::new(&std::fs::read(path).unwrap()[..]), image::PNG)
        .unwrap()
        .flipv()
        .to_rgba();
    let (width, height) = image.dimensions();
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        size: texture_extent,
        array_layer_count: 1,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
    });
    let temp_buf = device
        .create_buffer_mapped::<u8>(image.len(), wgpu::BufferUsage::COPY_SRC)
        .fill_from_slice(&image);
    encoder.copy_buffer_to_texture(
        wgpu::BufferCopyView {
            buffer: &temp_buf,
            offset: 0,
            row_pitch: 4 * texture_extent.width,
            image_height: texture_extent.height,
        },
        wgpu::TextureCopyView {
            texture: &texture,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        },
        texture_extent,
    );
    texture
}

fn main() {
    env_logger::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    log::warn!("NOTE: The window may freeze for a few seconds due to image loading being very slow in debug");
    cubism::core::set_core_logger(|s| log::warn!("CUBISM: {}", s));
    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res/Haru"]);

    // Create window
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(true)
        .with_min_inner_size(winit::dpi::LogicalSize {
            width: 128.0,
            height: 128.0,
        })
        .with_inner_size(winit::dpi::LogicalSize {
            width: 512.0,
            height: 512.0,
        })
        .build(&event_loop)
        .unwrap();
    let surface = wgpu::Surface::create(&window);

    let winit::dpi::PhysicalSize { width, height } = window.inner_size();

    let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        backends: wgpu::BackendBit::PRIMARY,
    })
    .unwrap();

    let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        extensions: wgpu::Extensions {
            anisotropic_filtering: false,
        },
        limits: wgpu::Limits::default(),
    });

    let mut sc_desc = wgpu::SwapChainDescriptor {
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        format: SWAPCHAIN_FORMAT,
        width,
        height,
        present_mode: wgpu::PresentMode::NoVsync,
    };
    let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
    window.request_redraw();

    // Load model3.json
    let haru_json = cubism::json::model::Model3::from_reader(
        File::open(&res_path.join("Haru.model3.json")).unwrap(),
    )
    .unwrap();

    // Load our cubism model
    let haru = cubism::model::UserModel::from_model3(&res_path, &haru_json).unwrap();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
    // Load textures
    let textures = haru_json
        .file_references
        .textures
        .iter()
        .map(|texpath| load_texture(&device, &mut encoder, &res_path.join(texpath)))
        .collect::<Vec<_>>();
    queue.submit(&[encoder.finish()]);

    let mut model_renderer = cubism_core_wgpu_renderer::Renderer::new(
        &haru,
        &device,
        &mut queue,
        sc_desc.format,
        textures,
    );
    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        use winit::dpi::PhysicalSize;
        use winit::event::{Event, WindowEvent};
        use winit::event_loop::ControlFlow;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // aka minimized
                WindowEvent::Resized(PhysicalSize {
                    width: 0,
                    height: 0,
                }) => (),
                WindowEvent::Resized(PhysicalSize { width, height }) => {
                    sc_desc.width = width;
                    sc_desc.height = height;
                    swap_chain = device.create_swap_chain(&surface, &sc_desc);
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let delta = now - last_frame;
                //let delta_s = delta.as_nanos() as f32 / 1e9;
                last_frame = now;
                let frame = swap_chain.get_next_texture();
                let mut encoder: wgpu::CommandEncoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color {
                            r: 0.8,
                            g: 0.5,
                            b: 0.4,
                            a: 1.0,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                model_renderer.draw_model(&device, &frame.view, &mut encoder, &haru);
                queue.submit(&[encoder.finish()]);
            }
            Event::MainEventsCleared => (),
            _ => (),
        }
    })
}
