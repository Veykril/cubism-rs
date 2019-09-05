use cubism::{expression::Expression, motion::Motion};
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

    // Show information
    println!("Press X to switch expression, press M to switch motion.");

    // load model
    use std::{iter::FromIterator, path::PathBuf};
    let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);

    let textures = load_textures(window);
    let mut haru =
        core::Model::from_bytes(&std::fs::read(&res_path.join("Haru/Haru.moc3")).unwrap()[..])
            .expect("Failed to load model.");
    let motions_path = &[
        "Haru/motions/haru_g_idle.motion3.json",
        "Haru/motions/haru_g_m15.motion3.json",
        "Haru/motions/haru_g_m06.motion3.json",
        "Haru/motions/haru_g_m09.motion3.json",
        "Haru/motions/haru_g_m20.motion3.json",
        "Haru/motions/haru_g_m26.motion3.json",
    ];
    let expressions_path = &[
        "Haru/expressions/F01.exp3.json",
        "Haru/expressions/F02.exp3.json",
        "Haru/expressions/F03.exp3.json",
        "Haru/expressions/F04.exp3.json",
        "Haru/expressions/F05.exp3.json",
        "Haru/expressions/F06.exp3.json",
        "Haru/expressions/F07.exp3.json",
        "Haru/expressions/F08.exp3.json",
    ];

    let mut motions_index: usize = 0;
    let mut motion = Motion::from_motion3_json(&res_path.join(motions_path[motions_index]))
        .expect("Failed to load motion.");
    let mut exp_index: usize = 0;
    let mut expression =
        Expression::from_exp3_json(&haru, &res_path.join(expressions_path[exp_index]))
            .expect("Failed to load expression.");

    // Play motion.
    motion.play();
    motion.set_looped(false); // just for cosmetic...

    // initialize renderer
    let mut renderer = Renderer::new();

    let mut g2d = Glium2d::new(opengl, window);

    while let Some(e) = window.next() {
        if let Some(v) = e.render_args() {
            use graphics::*;

            let viewport = v.draw_size;
            let mut target = window.draw();

            motion.update(&mut haru);
            expression.apply(&mut haru, 1.0);

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
            motion.tick(t.dt);
        }

        if let Some(b) = e.press_args() {
            if let Button::Keyboard(key) = b {
                match key {
                    Key::X => {
                        // switch expression
                        exp_index = exp_index + 1;
                        if exp_index == expressions_path.len() {
                            exp_index = 0;
                        }

                        println!("Switched expression to {}", expressions_path[exp_index]);

                        expression = Expression::from_exp3_json(
                            &haru,
                            &res_path.join(expressions_path[exp_index]),
                        )
                        .expect("Failed to load expression.");
                    },
                    Key::M => {
                        // switch motion
                        motions_index = motions_index + 1;
                        if motions_index == motions_path.len() {
                            motions_index = 0;
                        }

                        println!("Switched motion to {}", motions_path[motions_index]);

                        motion =
                            Motion::from_motion3_json(&res_path.join(motions_path[motions_index]))
                                .expect("Failed to load motion.");
                        motion.play();
                        motion.set_looped(false);
                    },
                    _ => {},
                }
            }
        }
    }
}
