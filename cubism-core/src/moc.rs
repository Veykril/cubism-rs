use std::{ffi::CStr, ptr, ptr::NonNull, slice};

use ffi::{csmMoc, csmModel};

use crate::{
    error::{MocError, MocResult},
    mem::AlignedMemory,
    ConstantFlags,
};

static INVALID_ID_STR: &str = "NON_UTF8_ID";

/// This represents a moc.
///
/// A moc should never exist without at least one model instance as it
/// owns the parameter, part and drawable ids as well as the minimum, maximum
/// and default parameter values of its [Model](./struct.Model.html).
///
/// All the data that can be accessed from a moc is static data that will not
/// change over the course of a programs lifetime.
#[derive(Debug)]
pub struct Moc {
    mem: AlignedMemory<csmMoc>,
    pub(in crate) part_ids: Box<[&'static str]>,
    pub(in crate) parameter_ids: Box<[&'static str]>,
    pub(in crate) drawable_ids: Box<[&'static str]>,
    param_def_val: NonNull<[f32]>,
    param_max_val: NonNull<[f32]>,
    param_min_val: NonNull<[f32]>,
    drawable_texture_indices: NonNull<[i32]>,
    drawable_constant_flags: NonNull<[ConstantFlags]>,
    drawable_indices: Box<[&'static [u16]]>,
    drawable_masks: Box<[&'static [i32]]>,
    drawable_vertex_counts: NonNull<[i32]>,
}

impl Moc {
    /// Returns the parameter names.
    #[inline]
    pub fn parameter_ids<'moc>(&'moc self) -> &[&'moc str] {
        &self.parameter_ids
    }

    /// Returns the part names.
    #[inline]
    pub fn part_ids<'moc>(&'moc self) -> &[&'moc str] {
        &self.part_ids
    }

    /// Returns the drawable names.
    #[inline]
    pub fn drawable_ids<'moc>(&'moc self) -> &[&'moc str] {
        &self.drawable_ids
    }

    /// Returns the parameter max values.
    #[inline]
    pub fn parameter_max(&self) -> &[f32] {
        unsafe { self.param_max_val.as_ref() }
    }

    /// Returns the parameter min values.
    #[inline]
    pub fn parameter_min(&self) -> &[f32] {
        unsafe { self.param_min_val.as_ref() }
    }

    /// Returns the parameter default values.
    #[inline]
    pub fn parameter_default(&self) -> &[f32] {
        unsafe { self.param_def_val.as_ref() }
    }

    /// Returns the number of parameters this moc has.
    #[inline]
    pub fn parameter_count(&self) -> usize {
        self.parameter_ids.len()
    }

    /// Returns the number of parts this moc has.
    #[inline]
    pub fn part_count(&self) -> usize {
        self.part_ids.len()
    }

    /// Returns the number of drawables this moc has.
    #[inline]
    pub fn drawable_count(&self) -> usize {
        self.drawable_ids.len()
    }

    /// Returns the texture indices of the drawables.
    #[inline]
    pub fn drawable_texture_indices(&self) -> &[i32] {
        unsafe { self.drawable_texture_indices.as_ref() }
    }

    /// Returns the [ConstantFlags](./struct.ConstantFlags.html).
    #[inline]
    pub fn drawable_constant_flags(&self) -> &[ConstantFlags] {
        unsafe { self.drawable_constant_flags.as_ref() }
    }

    /// Returns the vertex counts of each drawable.
    #[inline]
    pub fn drawable_vertex_counts(&self) -> &[i32] {
        unsafe { self.drawable_vertex_counts.as_ref() }
    }

    /// Returns the indices of the drawables.
    #[inline]
    pub fn drawable_indices<'moc>(&'moc self) -> &[&'moc [u16]] {
        &self.drawable_indices
    }

    /// Returns the masks of the drawables.
    #[inline]
    pub fn drawable_masks<'moc>(&'moc self) -> &[&'moc [i32]] {
        &self.drawable_masks
    }

    /// Returns true if this model is masked.
    #[inline]
    pub fn is_masked(&self) -> bool {
        self.drawable_masks.iter().any(|m| !m.is_empty())
    }

    /// Returns the raw [csmMoc](../cubism_core_sys/moc/struct.csmMoc.html) ptr
    #[inline]
    pub fn as_ptr(&self) -> *mut csmMoc {
        self.mem.as_ptr()
    }
}

impl Moc {
    unsafe fn new_moc(data: &[u8]) -> MocResult<AlignedMemory<csmMoc>> {
        let moc_ver = ffi::csmGetMocVersion(data.as_ptr() as _, data.len() as _);
        if ffi::csmGetLatestMocVersion() < moc_ver {
            Err(MocError::MocVersionMismatch(moc_ver))
        } else {
            let mem = AlignedMemory::alloc(data.len());
            ptr::copy_nonoverlapping(data.as_ptr(), mem.as_ptr() as *mut u8, data.len());
            let revived =
                ffi::csmReviveMocInPlace(mem.as_ptr() as _, mem.layout().size() as u32).is_null();
            if revived {
                Err(MocError::InvalidMocData)
            } else {
                Ok(mem)
            }
        }
    }

    pub(in crate) unsafe fn new(data: &[u8]) -> MocResult<(Self, AlignedMemory<csmModel>)> {
        let mem = Self::new_moc(data)?;
        let model = Self::init_new_model(mem.as_ptr());
        let model_ptr = model.as_ptr();

        let id_transform = |ptr, len| {
            slice::from_raw_parts_mut(ptr, len)
                .iter()
                .map(|ptr| CStr::from_ptr(*ptr).to_str().unwrap_or(INVALID_ID_STR))
        };

        let param_count = ffi::csmGetParameterCount(model_ptr) as usize;
        let part_count = ffi::csmGetPartCount(model_ptr) as usize;
        let drawable_count = ffi::csmGetDrawableCount(model_ptr) as usize;

        let indices = slice::from_raw_parts(ffi::csmGetDrawableIndices(model_ptr), drawable_count);
        let drawable_indices =
            slice::from_raw_parts(ffi::csmGetDrawableIndexCounts(model_ptr), drawable_count)
                .iter()
                .zip(indices)
                .map(|(c, indices)| slice::from_raw_parts(*indices, *c as usize))
                .collect();
        let masks = slice::from_raw_parts(ffi::csmGetDrawableMasks(model_ptr), drawable_count);
        let drawable_masks =
            slice::from_raw_parts(ffi::csmGetDrawableMaskCounts(model_ptr), drawable_count)
                .iter()
                .zip(masks)
                .map(|(c, masks)| slice::from_raw_parts(*masks, *c as usize))
                .collect();

        Ok((
            Moc {
                mem,
                part_ids: id_transform(ffi::csmGetPartIds(model_ptr), part_count).collect(),
                parameter_ids: id_transform(ffi::csmGetParameterIds(model_ptr), param_count)
                    .collect(),
                drawable_ids: id_transform(ffi::csmGetDrawableIds(model_ptr), drawable_count)
                    .collect(),
                param_def_val: NonNull::from(slice::from_raw_parts(
                    ffi::csmGetParameterDefaultValues(model_ptr),
                    param_count,
                )),
                param_max_val: NonNull::from(slice::from_raw_parts(
                    ffi::csmGetParameterMaximumValues(model_ptr),
                    param_count,
                )),
                param_min_val: NonNull::from(slice::from_raw_parts(
                    ffi::csmGetParameterMinimumValues(model_ptr),
                    param_count,
                )),
                drawable_texture_indices: NonNull::from(slice::from_raw_parts(
                    ffi::csmGetDrawableTextureIndices(model_ptr),
                    drawable_count,
                )),
                drawable_constant_flags: NonNull::from(slice::from_raw_parts(
                    ffi::csmGetDrawableConstantFlags(model_ptr) as _,
                    drawable_count,
                )),
                drawable_indices,
                drawable_masks,
                drawable_vertex_counts: NonNull::from(slice::from_raw_parts(
                    ffi::csmGetDrawableVertexCounts(model_ptr),
                    drawable_count,
                )),
            },
            model,
        ))
    }

    pub(in crate) unsafe fn init_new_model(moc: *const csmMoc) -> AlignedMemory<csmModel> {
        let model_size = ffi::csmGetSizeofModel(moc);
        let model_mem = AlignedMemory::alloc(model_size as usize);

        if ffi::csmInitializeModelInPlace(moc, model_mem.as_ptr() as *mut _, model_size).is_null() {
            unreachable!(
                "ffi::csmInitializeModelInPlace returned a null pointer, \
                 this shouldn't happen unless the alignment is incorrect"
            )
        } else {
            model_mem
        }
    }
}

unsafe impl Send for Moc {}
unsafe impl Sync for Moc {}
