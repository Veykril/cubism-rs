use glium::{glutin, texture::CompressedSrgbTexture2d, Display, Surface};
use imgui::{Condition, Context, FontConfig, FontSource, ImString};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use cubism::controller::ExpressionController;

use std::{
    fs::File,
    io::Cursor,
    iter::FromIterator,
    path::{Path, PathBuf},
    time::Instant,
};

fn init_imgui(imgui: &mut Context, display: &Display) -> (WinitPlatform, Renderer) {
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
    cubism::core::set_core_logger(|s| println!("{}", s));
    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res/Haru"]);

    // Create window and glutin context
    let mut events_loop = glutin::EventsLoop::new();
    let wb = glutin::WindowBuilder::new().with_title("cubism-rs debug app");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &events_loop).unwrap();

    // Init Imgui and its renderer
    let mut imgui = Context::create();
    let (mut platform, mut imgui_renderer) = init_imgui(&mut imgui, &display);

    // Load model3.json
    let haru_json = cubism::json::model::Model3::from_reader(
        File::open(&res_path.join("Haru.model3.json")).unwrap(),
    )
    .unwrap();

    // Load our cubism model
    let mut haru = cubism::model::UserModel::from_model3(&res_path, &haru_json).unwrap();
    let mut model_renderer =
        cubism_core_glium_renderer::Renderer::new(&display, haru.moc_arc()).unwrap();

    // Load textures
    let textures = haru_json
        .file_references
        .textures
        .iter()
        .map(|texpath| load_texture(&display, &res_path.join(texpath)))
        .collect::<Vec<_>>();

    let gl_window = display.gl_window();
    let window = gl_window.window();

    // Create ImStrings versions of the ids outside of the loop to prevent constant
    // reallocations
    let str_char_params = ImString::new("Params");
    let str_char_parts = ImString::new("Parts");
    let str_char_expressions = ImString::new("Expressions");
    let str_char_expressions_weight = ImString::new("Weight");
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
    let expr_con = haru.controller::<ExpressionController>().unwrap();
    let _exp_names = expr_con
        .names()
        .map(|id| ImString::new(id))
        .chain(std::iter::once(ImString::new("__None__")))
        .collect::<Vec<_>>();
    let exp_names = _exp_names.iter().map(|id| &*id).collect::<Vec<_>>();
    let mut exp_weight = 0.0;
    let mut current_expr = exp_names.len() as usize - 1;
    let mut last_frame = Instant::now();
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
        let delta_time = io.delta_time;
        let ui = imgui.frame();
        // Show sliders for all our parameters and parts
        ui.main_menu_bar(|| {
            ui.label_text(&imgui::im_str!("Delta: {}", delta_time), &ImString::new(""));
        });
        haru.load_parameters();
        imgui::Window::new(&str_char_params)
            .position([0.0, 20.0], Condition::Once)
            .size([362.0, 748.0], Condition::Once)
            .build(&ui, || {
                for (param, name) in haru.model_mut().parameters_mut().zip(&parameter_names) {
                    imgui::Slider::new(name, param.min_value..=param.max_value)
                        .build(&ui, param.value);
                }
            });
        imgui::Window::new(&str_char_parts)
            .position([662.0, 20.0], Condition::Once)
            .size([362.0, 480.0], Condition::Once)
            .build(&ui, || {
                for (opacity, name) in haru
                    .model_mut()
                    .part_opacities_mut()
                    .iter_mut()
                    .zip(&part_names)
                {
                    imgui::Slider::new(name, 0.0..=1.0).build(&ui, opacity);
                }
            });
        imgui::Window::new(&str_char_expressions)
            .position([362.0, 20.0], Condition::Once)
            .size([300.0, 100.0], Condition::Once)
            .build(&ui, || {
                let expr_con = haru.controller_mut::<ExpressionController>().unwrap();
                if imgui::ComboBox::new(&str_char_expressions).build_simple_string(
                    &ui,
                    &mut current_expr,
                    &*exp_names,
                ) {
                    expr_con.set_expression(exp_names[current_expr as usize].to_str());
                }
                imgui::Slider::new(&str_char_expressions_weight, 0.0..=1.0)
                    .build(&ui, &mut exp_weight);
                expr_con.set_expression_weight(exp_weight);
            });
        haru.save_parameters();
        haru.update(delta_time);

        // Start the rendering
        let mut target = display.draw();
        target.clear_color_srgb(0.70, 0.60, 0.60, 1.0);
        // Render our model
        model_renderer
            .draw_model(&mut target, &haru, &textures)
            .unwrap();

        // Render the imgui windows
        platform.prepare_render(&ui, window);
        let draw_data = ui.render();
        imgui_renderer
            .render(&mut target, draw_data)
            .expect("Rendering failed");

        // Finish the rendering
        target.finish().unwrap();
    }
}
