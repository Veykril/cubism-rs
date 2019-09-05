//! A Live2D Cubism renderer for [Piston](https://www.piston.rs/).
//!
//! Example:
//! ```
//! use cubism_core as core;
//! use cubism_core_piston2d_renderer::*;
//! use piston_window::*;
//! use sdl2_window::*;
//!
//! fn main() {
//!     let width = 600.0;
//!     let height = 850.0;
//!     let opengl = OpenGL::V3_2;
//!
//!     let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("Haru", [width, height])
//!         .exit_on_esc(true)
//!         .graphics_api(opengl)
//!         .samples(16)
//!         .build()
//!         .unwrap();
//!
//!     let mut t = window.create_texture_context();
//!
//!     // Initialize Live2D Cubism logger
//!     core::set_core_logger(|s| println!("{}", s));
//!
//!     // load model
//!     use std::{iter::FromIterator, path::PathBuf};
//!
//!     let res_path = PathBuf::from_iter(&[env!("CUBISM_CORE"), "Samples/Res"]);
//!
//!     let tex0 = Texture::from_path(
//!         &mut t,
//!         &res_path.join("Haru/Haru.2048/texture_00.png"),
//!         Flip::None,
//!         &TextureSettings::new(),
//!     )
//!     .expect("Failed to load texture");
//!     let tex1 = Texture::from_path(
//!         &mut t,
//!         &res_path.join("Haru/Haru.2048/texture_01.png"),
//!         Flip::None,
//!         &TextureSettings::new(),
//!     )
//!     .expect("Failed to load texture");
//!     let textures = [tex0, tex1];
//!
//!     let mut haru =
//!         core::Model::from_bytes(&std::fs::read(&res_path.join("Haru/Haru.moc3")).unwrap()[..])
//!             .expect("Failed to load model.");
//!
//!     // initialize renderer
//!     let mut renderer = Renderer::new();
//!
//!     while let Some(e) = window.next() {
//!         if let Some(v) = e.render_args() {
//!             let viewport = v.draw_size;
//!
//!             haru.update();
//!
//!             window.draw_2d(&e, |c, g, _d| {
//!                 let t = c
//!                     .transform
//!                     .trans(viewport[0] as f64 * 0.5, viewport[1] as f64 * 0.5)
//!                     .scale(0.2, 0.2);
//!                 clear([1.0, 1.0, 1.0, 1.0], g);
//!                 renderer.draw_model(g, t, &haru, &textures);
//!             });
//!         }
//!     }
//! }
//! ```

#![deny(missing_docs)]

use cubism_core::Model;
use graphics::{math::Matrix2d, DrawState, Graphics, ImageSize};

/// Live2D Cubism renderer for [Piston](https://www.piston.rs/).
pub struct Renderer {}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    /// Initializes a renderer.
    pub fn new() -> Renderer {
        Renderer {}
    }

    /// Draws a model.
    ///
    /// A model is drawn at (0, 0) pixel-by-pixel. In order to resize, relocate,
    /// or rotate, manipulate `transform`.
    pub fn draw_model<G, T>(
        &mut self,
        g: &mut G,
        transform: Matrix2d,
        model: &Model,
        textures: &[T],
    ) where
        G: Graphics<Texture = T>,
        T: ImageSize,
    {
        let mut sorted_draw_indices = vec![0; model.drawable_count()];

        for (idx, order) in model.drawable_render_orders().iter().enumerate() {
            sorted_draw_indices[*order as usize] = idx;
        }

        for draw_idx in sorted_draw_indices {
            self.draw_mesh(g, transform, model, draw_idx, textures, None);
        }
    }

    fn draw_mesh<G, T>(
        &mut self,
        g: &mut G,
        transform: Matrix2d,
        model: &Model,
        index: usize,
        textures: &[T],
        draw_state: Option<DrawState>,
    ) where
        G: Graphics<Texture = T>,
        T: ImageSize,
    {
        use cubism_core::{ConstantFlags, DynamicFlags};
        use graphics::draw_state::Blend;

        let opacity = model.drawable_opacities()[index];
        let dyn_flags = model.drawable_dynamic_flags()[index];

        if draw_state.is_none()
            && (opacity <= 0.0 || !dyn_flags.intersects(DynamicFlags::IS_VISIBLE))
        {
            return;
        }

        let blend_mode = model.drawable_constant_flags()[index];

        let draw_state = if let Some(draw_state) = draw_state {
            draw_state
        } else {
            // generate masks
            let masks = model.drawable_masks()[index];

            let state = if masks.is_empty() {
                DrawState::new_alpha()
            } else {
                let state = DrawState::new_clip();
                for i in masks {
                    self.draw_mesh(g, transform, model, *i as usize, textures, Some(state));
                }

                if blend_mode.intersects(ConstantFlags::IS_INVERTED_MASK) {
                    DrawState::new_outside()
                } else {
                    DrawState::new_inside()
                }
            };

            if blend_mode.intersects(ConstantFlags::BLEND_ADDITIVE) {
                state.blend(Blend::Lighter)
            } else if blend_mode.intersects(ConstantFlags::BLEND_MULTIPLICATIVE) {
                state.blend(Blend::Multiply)
            } else {
                state.blend(Blend::Alpha)
            }
        };

        let draw_state = &draw_state;

        // obtain pixel-per-unit information
        let (_, _, ppu) = model.canvas_info();

        let vtx_pos = model.drawable_vertex_positions(index);
        let vtx_uv = model.drawable_vertex_uvs(index);
        let idx_buffer = model.drawable_indices()[index];

        let tex = &textures[model.drawable_texture_indices()[index] as usize];

        let mut pos = Vec::with_capacity(idx_buffer.len());
        let mut uv = Vec::with_capacity(idx_buffer.len());

        use graphics::triangulation::{tx, ty};

        // extracts positions and UVs since Piston does not support index buffer objects
        for i in idx_buffer {
            let i = usize::from(*i);

            let [x, y] = vtx_pos[i];
            let (x, y) = (f64::from(x * ppu), f64::from(-y * ppu));

            let tp = [tx(transform, x, y), ty(transform, x, y)];
            pos.push(tp);
            uv.push([vtx_uv[i][0], 1.0 - vtx_uv[i][1]]);
        }

        let mut pos = &pos[0..];
        let mut uv = &uv[0..];

        // split by maximum vertex count and draw triangles
        use graphics::BACK_END_MAX_VERTEX_COUNT;
        while pos.len() >= BACK_END_MAX_VERTEX_COUNT {
            g.tri_list_uv(draw_state, &[1.0, 1.0, 1.0, opacity], tex, |f| {
                f(
                    &pos[0..BACK_END_MAX_VERTEX_COUNT],
                    &uv[0..BACK_END_MAX_VERTEX_COUNT],
                );
            });

            pos = &pos[BACK_END_MAX_VERTEX_COUNT..];
            uv = &uv[BACK_END_MAX_VERTEX_COUNT..];
        }

        // then, draw the rest
        g.tri_list_uv(draw_state, &[1.0, 1.0, 1.0, opacity], tex, |f| {
            f(&pos, &uv);
        });
    }
}
