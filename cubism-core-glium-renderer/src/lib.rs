use glium::{
    backend::{Context, Facade},
    index::{self, PrimitiveType},
    program::ProgramChooserCreationError,
    texture::{buffer_texture::TextureCreationError, CompressedSrgbTexture2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
    vertex::{self, VertexBuffer},
    Blend, DrawError, DrawParameters, IndexBuffer, Program, Surface,
};

use glium::{implement_vertex, program, uniform};

use core::fmt;
use std::{error::Error, rc::Rc};

use cubism_core::Model;

#[derive(Clone, Debug)]
pub enum RendererError {
    Vertex(vertex::BufferCreationError),
    Index(index::BufferCreationError),
    Program(ProgramChooserCreationError),
    Texture(TextureCreationError),
    Draw(DrawError),
}

impl Error for RendererError {}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::RendererError::*;
        match *self {
            Vertex(_) => write!(f, "Vertex buffer creation failed"),
            Index(_) => write!(f, "Index buffer creation failed"),
            Program(ref e) => write!(f, "Program creation failed: {}", e),
            Texture(_) => write!(f, "Texture creation failed"),
            Draw(ref e) => write!(f, "Drawing failed: {}", e),
        }
    }
}

impl From<vertex::BufferCreationError> for RendererError {
    fn from(e: vertex::BufferCreationError) -> RendererError {
        RendererError::Vertex(e)
    }
}

impl From<index::BufferCreationError> for RendererError {
    fn from(e: index::BufferCreationError) -> RendererError {
        RendererError::Index(e)
    }
}

impl From<ProgramChooserCreationError> for RendererError {
    fn from(e: ProgramChooserCreationError) -> RendererError {
        RendererError::Program(e)
    }
}

impl From<TextureCreationError> for RendererError {
    fn from(e: TextureCreationError) -> RendererError {
        RendererError::Texture(e)
    }
}

impl From<DrawError> for RendererError {
    fn from(e: DrawError) -> RendererError {
        RendererError::Draw(e)
    }
}

#[derive(Copy, Clone)]
struct Vertex {
    a_pos: [f32; 2],
    a_tex_coords: [f32; 2],
}
implement_vertex!(Vertex, a_pos, a_tex_coords);

pub struct Renderer {
    ctx: Rc<Context>,
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    mvp: mint::ColumnMatrix4<f32>,
}

impl Renderer {
    pub fn new<F: Facade>(facade: &F) -> Result<Self, RendererError> {
        let program = compile_default_program(facade)?;
        let vertex_buffer = VertexBuffer::dynamic(
            facade,
            &[Vertex {
                a_pos: [0.0, 0.0],
                a_tex_coords: [0.0, 0.0],
            }; 256],
        )?;
        let index_buffer = IndexBuffer::dynamic(facade, PrimitiveType::TrianglesList, &[0; 256])?;
        Ok(Renderer {
            ctx: Rc::clone(facade.get_context()),
            program,
            vertex_buffer,
            index_buffer,
            mvp: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ]
            .into(),
        })
    }

    pub fn draw_model<T: Surface>(
        &mut self,
        target: &mut T,
        model: &Model,
        textures: &[CompressedSrgbTexture2d],
    ) -> Result<(), RendererError> {
        let mut sorted_draw_indices = vec![0; model.drawable_count()];
        for (idx, order) in model.drawable_render_orders().iter().enumerate() {
            sorted_draw_indices[*order as usize] = idx;
        }

        for draw_idx in sorted_draw_indices {
            self.draw_mesh(target, model, draw_idx, textures)?;
        }
        Ok(())
    }

    fn draw_mesh<T: Surface>(
        &mut self,
        target: &mut T,
        model: &Model,
        index: usize,
        textures: &[CompressedSrgbTexture2d],
    ) -> Result<(), RendererError> {
        let opacity = model.drawable_opacities()[index];
        if opacity <= 0.0 {
            return Ok(());
        }
        let vtx_pos = model.drawable_vertex_positions(index);
        let vtx_uv = model.drawable_vertex_uvs(index);
        let mut vtx_buffer = Vec::with_capacity(vtx_pos.len());
        for i in 0..vtx_pos.len() {
            let vtx_pos = vtx_pos[i];
            let vtx_uv = vtx_uv[i];
            vtx_buffer.push(Vertex {
                a_pos: [vtx_pos[0], vtx_pos[1]],
                a_tex_coords: [vtx_uv[0], vtx_uv[1]],
            });
        }
        let idx_buffer = Vec::from(model.drawable_indices(index));
        self.upload_vertex_buffer(&vtx_buffer)?;
        self.upload_index_buffer(&idx_buffer)?;

        let tex = &textures[model.drawable_texture_indices()[index] as usize];
        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniform! {
                    u_mvp: Into::<[[f32; 4]; 4]>::into(self.mvp),
                    u_tex0: tex.sampled()
                        .minify_filter(MinifySamplerFilter::Linear)
                        .magnify_filter(MagnifySamplerFilter::Linear)
                },
                &DrawParameters {
                    blend: Blend::alpha_blending(),
                    ..DrawParameters::default()
                },
            )
            .map_err(|e| e.into())
    }

    fn upload_vertex_buffer(&mut self, vtx_buffer: &[Vertex]) -> Result<(), RendererError> {
        if self.vertex_buffer.len() != vtx_buffer.len() {
            self.vertex_buffer = VertexBuffer::dynamic(&self.ctx, vtx_buffer)?;
        } else {
            self.vertex_buffer.write(vtx_buffer);
        }
        Ok(())
    }

    fn upload_index_buffer(&mut self, idx_buffer: &[u16]) -> Result<(), RendererError> {
        if self.index_buffer.len() != idx_buffer.len() {
            self.index_buffer =
                IndexBuffer::dynamic(&self.ctx, PrimitiveType::TrianglesList, idx_buffer)?;
        } else {
            self.index_buffer.write(idx_buffer);
        }
        Ok(())
    }

    pub fn mvp(&self) -> mint::ColumnMatrix4<f32> {
        self.mvp
    }

    pub fn mvp_mut(&mut self) -> &mut mint::ColumnMatrix4<f32> {
        &mut self.mvp
    }

    pub fn set_mvp<M: Into<mint::ColumnMatrix4<f32>>>(&mut self, mat: M) {
        self.mvp = mat.into();
    }
}

fn compile_default_program<F: Facade>(facade: &F) -> Result<Program, ProgramChooserCreationError> {
    program!(
        facade,
        330 => {
            vertex: include_str!("shader/330.vert"),
            fragment: include_str!("shader/330.frag"),
            outputs_srgb: true,
        },
    )
}
