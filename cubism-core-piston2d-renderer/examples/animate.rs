use cubism::motion::Motion;
use cubism_core as core;
use cubism_core_piston2d_renderer::*;
use glium::backend::Facade;
use glium_graphics::{Flip, Glium2d, GliumWindow, OpenGL, Texture, TextureSettings};
use piston::{input::*, window::WindowSettings};
use sdl2_window::*;

fn load_textures<F: Facade>(window: &mut F) -> [Texture; 2] {
    use std::{iter::FromIterator, path::PathBuf};

    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);

    let tex0 = Texture::from_path(
        window,
        &res_path.join("Haru/Haru.2048/texture_00.png"),
        Flip::None,
        &TextureSettings::new(),
    )
    .expect("Failed to load texture");
    let tex1 = Texture::from_path(
        window,
        &res_path.join("Haru/Haru.2048/texture_01.png"),
        Flip::None,
        &TextureSettings::new(),
    )
    .expect("Failed to load texture");

    [tex0, tex1]
}

fn main() {
    let width = 600.0;
    let height = 850.0;
    let opengl = OpenGL::V3_2;

    let ref mut window: GliumWindow<Sdl2Window> =
        WindowSettings::new("Animation Test: Haru", [width, height])
            .exit_on_esc(true)
            .graphics_api(opengl)
            .samples(4)
            .build()
            .unwrap();

    // Initialize Live2D Cubism logger
    core::set_core_logger(|s| println!("{}", s));

    // load model
    use std::{iter::FromIterator, path::PathBuf};
    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);

    let textures = load_textures(window);
    let mut haru =
        core::Model::from_bytes(&std::fs::read(&res_path.join("Haru/Haru.moc3")).unwrap()[..])
            .expect("Failed to load model.");
    let mut motion_idle =
        Motion::from_motion3_json(&res_path.join("Haru/motions/haru_g_idle.motion3.json"))
            .expect("Failed to load motion.");
    motion_idle.play();

    // initialize renderer
    let mut renderer = Renderer::new();

    let mut g2d = Glium2d::new(opengl, window);

    while let Some(e) = window.next() {
        if let Some(v) = e.render_args() {
            use graphics::*;

            let viewport = v.draw_size;
            let mut target = window.draw();

            motion_idle.update(&mut haru).unwrap();
            haru.update();

            g2d.draw(&mut target, v.viewport(), |c, g| {
                let t = c
                    .transform
                    .trans(viewport[0] as f64 * 0.5, viewport[1] as f64 * 0.5)
                    .scale(0.2, 0.2);
                clear([1.0, 1.0, 1.0, 1.0], g);
                renderer.draw_model(g, t, &haru, &textures);
            });

            target.finish().unwrap();
        }
        if let Some(t) = e.update_args() {
            motion_idle.tick(t.dt);
        }
    }
}
