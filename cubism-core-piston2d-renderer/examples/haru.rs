use cubism_core as core;
use cubism_core_piston2d_renderer::*;
use piston_window::*;
use sdl2_window::*;

fn main() {
    let width = 600.0;
    let height = 850.0;
    let opengl = OpenGL::V3_2;

    let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("Haru", [width, height])
        .exit_on_esc(true)
        .graphics_api(opengl)
        .samples(16)
        .build()
        .unwrap();

    let mut t = window.create_texture_context();

    // Initialize Live2D Cubism logger
    core::set_core_logger(|s| println!("{}", s));

    // load model
    use std::{iter::FromIterator, path::PathBuf};

    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);

    let tex0 = Texture::from_path(
        &mut t,
        &res_path.join("Haru/Haru.2048/texture_00.png"),
        Flip::None,
        &TextureSettings::new(),
    )
    .expect("Failed to load texture");
    let tex1 = Texture::from_path(
        &mut t,
        &res_path.join("Haru/Haru.2048/texture_01.png"),
        Flip::None,
        &TextureSettings::new(),
    )
    .expect("Failed to load texture");
    let textures = [tex0, tex1];

    let mut haru =
        core::Model::from_bytes(&std::fs::read(&res_path.join("Haru/Haru.moc3")).unwrap()[..])
            .expect("Failed to load model.");

    // initialize renderer
    let mut renderer = Renderer::new();

    while let Some(e) = window.next() {
        if let Some(v) = e.render_args() {
            let viewport = v.draw_size;

            haru.update();

            window.draw_2d(&e, |c, g, _d| {
                let t = c
                    .transform
                    .trans(viewport[0] as f64 * 0.5, viewport[1] as f64 * 0.5)
                    .scale(0.2, 0.2);
                clear([1.0, 1.0, 1.0, 1.0], g);
                renderer.draw_model(g, t, &haru, &textures);
            });
        }
    }
}
