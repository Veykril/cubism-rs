use std::os::raw::{c_char, c_float, c_int, c_uchar, c_uint, c_ushort, c_void};

use crate::{csmVector2, moc::csmMoc};

pub const csmAlignofModel: usize = 16;

pub const csmBlendAdditive: csmFlags = 1 << 0;
pub const csmBlendMultiplicative: csmFlags = 1 << 1;
pub const csmIsDoubleSided: csmFlags = 1 << 2;
pub const csmIsInvertedMask: csmFlags = 1 << 3;

pub const csmIsVisible: csmFlags = 1 << 0;
pub const csmVisibilityDidChange: csmFlags = 1 << 1;
pub const csmOpacityDidChange: csmFlags = 1 << 2;
pub const csmDrawOrderDidChange: csmFlags = 1 << 3;
pub const csmRenderOrderDidChange: csmFlags = 1 << 4;
pub const csmVertexPositionsDidChange: csmFlags = 1 << 5;

pub type csmFlags = c_uchar;

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct csmModel {
    _unused: [u16; 0],
}

extern "C" {
    pub fn csmGetSizeofModel(moc: *const csmMoc) -> c_uint;
    pub fn csmInitializeModelInPlace(
        moc: *const csmMoc,
        aligned_address: *mut c_void,
        size: c_uint,
    ) -> *mut csmModel;
    pub fn csmUpdateModel(model: *mut csmModel);
    pub fn csmReadCanvasInfo(
        model: *const csmModel,
        outSizeInPixels: *mut csmVector2,
        outOriginalInPixels: *mut csmVector2,
        outPixelsPerUnit: *mut c_float,
    );

    pub fn csmGetParameterCount(model: *const csmModel) -> c_int;
    pub fn csmGetParameterIds(model: *const csmModel) -> *mut *const c_char;
    pub fn csmGetParameterMinimumValues(model: *const csmModel) -> *const c_float;
    pub fn csmGetParameterMaximumValues(model: *const csmModel) -> *const c_float;
    pub fn csmGetParameterDefaultValues(model: *const csmModel) -> *const c_float;
    pub fn csmGetParameterValues(model: *mut csmModel) -> *mut c_float;

    pub fn csmGetPartCount(model: *const csmModel) -> c_int;
    pub fn csmGetPartIds(model: *const csmModel) -> *mut *const c_char;
    pub fn csmGetPartOpacities(model: *mut csmModel) -> *mut c_float;
    pub fn csmGetPartParentPartIndices(model: *const csmModel) -> *const c_int;

    pub fn csmGetDrawableCount(model: *const csmModel) -> c_int;
    pub fn csmGetDrawableIds(model: *const csmModel) -> *mut *const c_char;
    pub fn csmGetDrawableConstantFlags(model: *const csmModel) -> *const csmFlags;
    pub fn csmGetDrawableDynamicFlags(model: *const csmModel) -> *const csmFlags;
    pub fn csmGetDrawableTextureIndices(model: *const csmModel) -> *const c_int;
    pub fn csmGetDrawableDrawOrders(model: *const csmModel) -> *const c_int;
    pub fn csmGetDrawableRenderOrders(model: *const csmModel) -> *const c_int;
    pub fn csmGetDrawableOpacities(model: *const csmModel) -> *const c_float;
    pub fn csmGetDrawableMaskCounts(model: *const csmModel) -> *const c_int;
    pub fn csmGetDrawableMasks(model: *const csmModel) -> *mut *const c_int;
    pub fn csmGetDrawableVertexCounts(model: *const csmModel) -> *const c_int;
    pub fn csmGetDrawableVertexPositions(model: *const csmModel) -> *mut *const csmVector2;
    pub fn csmGetDrawableVertexUvs(model: *const csmModel) -> *mut *const csmVector2;
    pub fn csmGetDrawableIndexCounts(model: *const csmModel) -> *const c_int;
    pub fn csmGetDrawableIndices(model: *const csmModel) -> *mut *const c_ushort;
    pub fn csmResetDrawableDynamicFlags(model: *mut csmModel);
}

#[test]
fn model_alignment() {
    assert_eq!(::std::mem::align_of::<csmModel>(), csmAlignofModel);
}
