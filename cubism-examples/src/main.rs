use glium::{glutin, texture::CompressedSrgbTexture2d, Display, Surface};
use imgui::{Condition, Context, FontConfig, FontSource, ImString};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use std::{
    io::Cursor,
    iter::FromIterator,
    path::{Path, PathBuf},
    time::Instant,
};

fn init_imgui(imgui: &mut Context, display: &Display) -> (WinitPlatform, Renderer) {
    // imgui
    imgui.set_ini_filename(None);
    let mut platform = WinitPlatform::init(imgui);
    let gl_window = display.gl_window();
    platform.attach_window(imgui.io_mut(), &gl_window.window(), HiDpiMode::Rounded);

    let hidpi_factor = platform.hidpi_factor();
    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig {
            size_pixels: (13.0 * hidpi_factor) as f32,
            ..FontConfig::default()
        }),
    }]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
    (
        platform,
        Renderer::init(imgui, display).expect("Failed to initialize renderer"),
    )
}

fn load_texture(display: &Display, path: &Path) -> CompressedSrgbTexture2d {
    let image = image::load(Cursor::new(&std::fs::read(path).unwrap()[..]), image::PNG)
        .unwrap()
        .to_rgba();
    let image_dimensions = image.dimensions();
    let image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    CompressedSrgbTexture2d::new(display, image).unwrap()
}

fn main() {
    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);

    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    let mut imgui = Context::create();
    let (mut platform, mut imgui_renderer) = init_imgui(&mut imgui, &display);

    let tex0 = load_texture(&display, &res_path.join("Haru/Haru.2048/texture_00.png"));
    let tex1 = load_texture(&display, &res_path.join("Haru/Haru.2048/texture_01.png"));
    let textures = [tex0, tex1];

    let mut haru = cubism::core::Model::from_bytes(
        &std::fs::read(&res_path.join("Haru/Haru.moc3")).unwrap()[..],
    )
    .unwrap();
    let mut model_renderer = cubism_core_glium_renderer::Renderer::new(&display).unwrap();

    let mut last_frame = Instant::now();
    let gl_window = display.gl_window();
    let window = gl_window.window();
    let str_char_params = ImString::new("CharParams");
    let str_char_parts = ImString::new("CharParts");
    let parameter_names = haru
        .parameter_ids()
        .iter()
        .map(|id| ImString::new(*id))
        .collect::<Vec<_>>();
    let part_names = haru
        .part_ids()
        .iter()
        .map(|id| ImString::new(*id))
        .collect::<Vec<_>>();
    loop {
        let mut exit = false;
        events_loop.poll_events(|event| {
            platform.handle_event(imgui.io_mut(), &window, &event);
            if let glutin::Event::WindowEvent {
                event: glutin::WindowEvent::CloseRequested,
                ..
            } = event
            {
                exit = true;
            }
        });
        if exit {
            break;
        }

        let io = imgui.io_mut();
        platform
            .prepare_frame(io, &window)
            .expect("Failed to start frame");
        last_frame = io.update_delta_time(last_frame);
        let ui = imgui.frame();
        ui.window(&str_char_params)
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                for idx in 0..haru.parameter_count() {
                    let min = haru.parameter_min()[idx];
                    let max = haru.parameter_max()[idx];
                    ui.slider_float(
                        &parameter_names[idx],
                        &mut haru.parameter_values_mut()[idx],
                        min,
                        max,
                    )
                    .build();
                }
            });
        ui.window(&str_char_parts)
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(|| {
                for (part, opacity) in part_names.iter().zip(haru.part_opacities_mut()) {
                    ui.slider_float(part, opacity, 0.0, 1.0).build();
                }
            });
        haru.update();

        // drawing a frame
        let mut target = display.draw();
        target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
        model_renderer
            .draw_model(&mut target, &haru, &textures)
            .unwrap();

        platform.prepare_render(&ui, window);
        let draw_data = ui.render();
        imgui_renderer
            .render(&mut target, draw_data)
            .expect("Rendering failed");
        target.finish().unwrap();
    }
}
