pub mod dummy_hdm;
pub mod hardware_data_manager;
mod saf_raw;

use libc::c_void;

pub fn bn_create() -> *mut c_void {
    unsafe {
        let mut ph_bin = std::ptr::null_mut();
        saf_raw::binauraliser_create(std::ptr::addr_of_mut!(ph_bin));
        return ph_bin;
    }
}

pub fn bn_destroy(mut ph_bin: *mut c_void) {
    unsafe {
        saf_raw::binauraliser_destroy(std::ptr::addr_of_mut!(ph_bin));
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
        let res = bn_create();
        bn_destroy(res);
    }
}
