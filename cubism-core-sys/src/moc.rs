use std::os::raw::{c_uint, c_void};

pub const csmAlignofMoc: usize = 64;

pub const csmMocVersion_Unknown: csmMocVersion = 0;
pub const csmMocVersion_30: csmMocVersion = 1;
pub const csmMocVersion_33: csmMocVersion = 2;
pub const csmMocVersion_40: csmMocVersion = 3;

pub type csmMocVersion = c_uint;

#[repr(C, align(64))]
#[derive(Copy, Clone, Debug)]
pub struct csmMoc {
    _unused: [u64; 0],
}

extern "C" {
    pub fn csmGetLatestMocVersion() -> csmMocVersion;
    pub fn csmGetMocVersion(address: *const c_void, size: c_uint) -> csmMocVersion;
    pub fn csmReviveMocInPlace(aligned_address: *mut c_void, size: c_uint) -> *mut csmMoc;
}

#[test]
fn alignment() {
    assert_eq!(::std::mem::align_of::<csmMoc>(), csmAlignofMoc);
}
