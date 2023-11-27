use crate::saf_raw;

use libc::c_void;

pub fn bn_create() -> *mut c_void {
    unsafe {
        let mut h_bin = std::ptr::null_mut();
        saf_raw::binauraliserNF_create(std::ptr::addr_of_mut!(h_bin));
        return h_bin;
    }
}

pub fn bn_destroy(h_bin: &mut *mut c_void) {
    unsafe {
        saf_raw::binauraliserNF_destroy(h_bin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ph_bin_is_created() {
        let res = bn_create();
        assert!(!res.is_null());
    }

    #[test]
    fn ph_bin_is_destroyed() {
        let mut res = bn_create();
        bn_destroy(&mut res);
        assert!(res.is_null());
    }
}
