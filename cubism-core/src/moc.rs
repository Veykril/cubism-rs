use core::{ptr, slice};
use std::{ffi::CStr, ptr::NonNull};

use ffi::{csmMoc, csmModel};

use crate::{
    error::{CubismError, CubismResult},
    mem::AlignedMemory,
};

static INVALID_ID_STR: &str = "NON_UTF8_ID";

/// This represents a moc.
///
/// A moc should never exist without at least one model instance as it
/// owns the parameter, part and drawable ids as well as the minimum, maximum
/// and default parameter values of its [Model](./struct.Model.html).
#[derive(Debug)]
pub struct Moc {
    mem: AlignedMemory<csmMoc>,
    part_ids: Vec<&'static str>,
    param_ids: Vec<&'static str>,
    drawable_ids: Vec<&'static str>,
    param_def_val: NonNull<[f32]>,
    param_max_val: NonNull<[f32]>,
    param_min_val: NonNull<[f32]>,
}

impl Moc {
    /// Returns the part names.
    #[inline]
    pub fn part_ids<'moc>(&'moc self) -> &[&'moc str] {
        &self.part_ids
    }

    /// Returns the parameter names.
    #[inline]
    pub fn parameter_ids<'moc>(&'moc self) -> &[&'moc str] {
        &self.param_ids
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
        self.param_ids.len()
    }

    /// Returns the number of parts this moc has.
    #[inline]
    pub fn part_count(&self) -> usize {
        self.part_ids.len()
    }

    /// Returns the raw [csmMoc](../cubism_core_sys/moc/struct.csmMoc.html) ptr
    #[inline]
    pub fn as_ptr(&self) -> *mut csmMoc {
        self.mem.as_ptr()
    }
}

impl Moc {
    unsafe fn new_moc(data: &[u8]) -> CubismResult<AlignedMemory<csmMoc>> {
        let moc_ver = ffi::csmGetMocVersion(data.as_ptr() as _, data.len() as _);
        if ffi::csmGetLatestMocVersion() < moc_ver {
            Err(CubismError::MocVersionMismatch(moc_ver))
        } else {
            let mem = AlignedMemory::alloc(data.len());
            ptr::copy_nonoverlapping(data.as_ptr(), mem.as_ptr() as *mut u8, data.len());
            let revived =
                ffi::csmReviveMocInPlace(mem.as_ptr() as _, mem.layout().size() as u32).is_null();
            if revived {
                Err(CubismError::InvalidMocData)
            } else {
                Ok(mem)
            }
        }
    }

    pub(in crate) unsafe fn new(data: &[u8]) -> CubismResult<(Self, AlignedMemory<csmModel>)> {
        let mem = Self::new_moc(data)?;
        let dangling = NonNull::new_unchecked(slice::from_raw_parts_mut(0x1 as *mut f32, 0));
        let mut this = Moc {
            mem,
            part_ids: Vec::new(),
            param_ids: Vec::new(),
            drawable_ids: Vec::new(),
            param_def_val: dangling,
            param_max_val: dangling,
            param_min_val: dangling,
        };
        let model = this.init_new_model();
        this.init_ids(&model);
        Ok((this, model))
    }

    unsafe fn init_ids(&mut self, model: &AlignedMemory<csmModel>) {
        let model_ptr = model.as_ptr();
        let id_transform = |ptr, len| {
            slice::from_raw_parts_mut(ptr, len)
                .iter()
                .map(|ptr| CStr::from_ptr(*ptr).to_str().unwrap_or(INVALID_ID_STR))
        };

        let param_count = ffi::csmGetParameterCount(model_ptr) as usize;
        let param_ids = ffi::csmGetParameterIds(model_ptr);
        self.param_ids = id_transform(param_ids, param_count).collect();
        let part_count = ffi::csmGetPartCount(model_ptr) as usize;
        let part_ids = ffi::csmGetPartIds(model_ptr);
        self.part_ids = id_transform(part_ids, part_count).collect();
        let drawable_count = ffi::csmGetDrawableCount(model_ptr) as usize;
        let drawable_ids = ffi::csmGetDrawableIds(model_ptr);
        self.drawable_ids = id_transform(drawable_ids, drawable_count).collect();

        self.param_def_val = NonNull::from(slice::from_raw_parts(
            ffi::csmGetParameterDefaultValues(model_ptr),
            param_count,
        ));
        self.param_max_val = NonNull::from(slice::from_raw_parts(
            ffi::csmGetParameterMaximumValues(model_ptr),
            param_count,
        ));
        self.param_min_val = NonNull::from(slice::from_raw_parts(
            ffi::csmGetParameterMinimumValues(model_ptr),
            param_count,
        ));
    }

    pub(in crate) unsafe fn init_new_model(&self) -> AlignedMemory<csmModel> {
        let model_size = ffi::csmGetSizeofModel(self.mem.as_ptr());
        let model_mem = AlignedMemory::alloc(model_size as usize);

        if ffi::csmInitializeModelInPlace(
            self.mem.as_ptr(),
            model_mem.as_ptr() as *mut _,
            model_size,
        )
        .is_null()
        {
            unreachable!(
                "ffi::csmInitializeModelInPlace returned a null pointer, \
                 this shouldn't happen unless the alignment is incorrect"
            )
        } else {
            model_mem
        }
    }
}
