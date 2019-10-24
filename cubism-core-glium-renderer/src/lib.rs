use glium::{
    backend::Facade,
    index::{self, PrimitiveType},
    program::ProgramCreationInput::SourceCode,
    texture::{buffer_texture::TextureCreationError, CompressedSrgbTexture2d},
    uniforms::{MagnifySamplerFilter, MinifySamplerFilter},
    vertex::{self, VertexBuffer},
    BackfaceCullingMode, DrawError, DrawParameters, IndexBuffer, Program, ProgramCreationError,
    Surface,
};

use glium::{implement_vertex, uniform};

use std::{error::Error, fmt, ptr, sync::Arc};

use cubism_core::{ConstantFlags, Drawable, DynamicFlags, Moc, Model};

#[derive(Clone, Debug)]
pub enum RendererError {
    MocMismatch,
    Vertex(vertex::BufferCreationError),
    Index(index::BufferCreationError),
    Program(ProgramCreationError),
    Texture(TextureCreationError),
    Draw(DrawError),
}

impl Error for RendererError {}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::RendererError::*;
        match *self {
            MocMismatch => write!(
                f,
                "renderer received different moc than what it has been initialized with"
            ),
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

impl From<ProgramCreationError> for RendererError {
    fn from(e: ProgramCreationError) -> RendererError {
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

#[inline]
fn create_program<F: Facade>(
    facade: &F,
    vertex_shader: &str,
    fragment_shader: &str,
) -> Result<Program, ProgramCreationError> {
    Program::new(
        facade,
        SourceCode {
            vertex_shader,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            geometry_shader: None,
            fragment_shader,
            transform_feedback_varyings: None,
            outputs_srgb: true,
            uses_point_size: false,
        },
    )
}

#[derive(Copy, Clone)]
struct Vertex {
    in_pos: [f32; 2],
    in_tex_coords: [f32; 2],
}
implement_vertex!(Vertex, in_pos, in_tex_coords);

pub struct Renderer {
    moc: Arc<Moc>,
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffers: Vec<IndexBuffer<u16>>,
    mvp: mint::ColumnMatrix4<f32>,
}

impl Renderer {
    pub fn new<F: Facade>(facade: &F, moc: Arc<Moc>) -> Result<Self, RendererError> {
        let program = create_program(
            facade,
            include_str!("shader/normal.vert"),
            include_str!("shader/normal.frag"),
        )?;
        let vertex_buffer = VertexBuffer::empty_dynamic(
            facade,
            moc.drawable_vertex_counts()
                .iter()
                .max()
                .copied()
                .unwrap_or_default() as usize,
        )?;
        let index_buffers = moc
            .drawable_indices()
            .iter()
            .map(|indices| IndexBuffer::immutable(facade, PrimitiveType::TrianglesList, indices))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Renderer {
            moc,
            program,
            vertex_buffer,
            index_buffers,
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
        if !ptr::eq(model.moc(), &*self.moc) {
            Err(RendererError::MocMismatch)
        } else {
            let mut drawables: Vec<_> = model.drawables().collect();
            drawables.sort_unstable_by_key(|d| d.render_order);
            // pass by ref or value? Drawable is quite a big structure
            for drawable in &drawables {
                self.draw_drawable(target, drawable, textures)?;
            }
            Ok(())
        }
    }

    fn draw_drawable<T: Surface>(
        &mut self,
        target: &mut T,
        drawable: &Drawable,
        textures: &[CompressedSrgbTexture2d],
    ) -> Result<(), RendererError> {
        let dflags = drawable.dynamic_flags;
        if drawable.opacity <= 0.0 || !dflags.intersects(DynamicFlags::IS_VISIBLE) {
            return Ok(());
        }
        let vtx_pos = drawable.vertex_positions;
        let vtx_uv = drawable.vertex_uvs;
        let vtx_buffer = vtx_pos
            .iter()
            .zip(vtx_uv)
            .map(|(pos, uv)| Vertex {
                in_pos: [pos[0], pos[1]],
                in_tex_coords: [uv[0], uv[1]],
            })
            .collect::<Vec<_>>();
        self.vertex_buffer
            .slice(0..vtx_pos.len())
            .unwrap()
            .write(&vtx_buffer);

        let cflags = drawable.constant_flags;
        let blend = if cflags.intersects(ConstantFlags::BLEND_MULTIPLICATIVE) {
            blend::MULTIPLICATIVE
        } else if cflags.intersects(ConstantFlags::BLEND_ADDITIVE) {
            blend::ADDITIVE
        } else {
            blend::NORMAL
        };
        let backface_culling = if cflags.intersects(ConstantFlags::IS_DOUBLE_SIDED) {
            BackfaceCullingMode::CullingDisabled
        } else {
            BackfaceCullingMode::CullCounterClockwise
        };

        let tex = &textures[drawable.texture_index as usize];
        target
            .draw(
                &self.vertex_buffer,
                &self.index_buffers[drawable.index],
                &self.program,
                &uniform! {
                    u_mvp: Into::<[[f32; 4]; 4]>::into(self.mvp),
                    us_tex0: tex.sampled()
                        .minify_filter(MinifySamplerFilter::Linear)
                        .magnify_filter(MagnifySamplerFilter::Linear)
                },
                &DrawParameters {
                    blend,
                    backface_culling,
                    ..DrawParameters::default()
                },
            )
            .map_err(|e| e.into())
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

mod blend {
    use glium::{Blend, BlendingFunction as BF, LinearBlendingFactor as LBF};

    pub const NORMAL: Blend = Blend {
        color: BF::Addition {
            source: LBF::One,
            destination: LBF::OneMinusSourceAlpha,
        },
        alpha: BF::Addition {
            source: LBF::One,
            destination: LBF::OneMinusSourceAlpha,
        },
        constant_value: (0.0, 0.0, 0.0, 0.0),
    };

    pub const ADDITIVE: Blend = Blend {
        color: BF::Addition {
            source: LBF::One,
            destination: LBF::One,
        },
        alpha: BF::Addition {
            source: LBF::Zero,
            destination: LBF::One,
        },
        constant_value: (0.0, 0.0, 0.0, 0.0),
    };

    pub const MULTIPLICATIVE: Blend = Blend {
        color: BF::Addition {
            source: LBF::DestinationColor,
            destination: LBF::OneMinusSourceAlpha,
        },
        alpha: BF::Addition {
            source: LBF::Zero,
            destination: LBF::One,
        },
        constant_value: (0.0, 0.0, 0.0, 0.0),
    };
}
