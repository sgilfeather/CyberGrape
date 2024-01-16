//! A safe api into the Spatial Audio Framework.

use std::ops::Index;

use crate::saf_raw;

use libc::c_void;

pub trait Binauraliser {
    fn process<B>(&self, buffers: &[(Position, B)]) -> Vec<B>
    where
        B: Index<usize, Output = f32>;
}

pub struct Position {}

pub struct BinauraliserNF {
    h_bin: *mut c_void,
}

impl BinauraliserNF {
    pub fn new() -> Self {
        let mut h_bin = std::ptr::null_mut();
        unsafe {
            saf_raw::binauraliserNF_create(std::ptr::addr_of_mut!(h_bin));
        }
        BinauraliserNF { h_bin }
    }
}

impl Drop for BinauraliserNF {
    fn drop(&mut self) {
        unsafe {
            saf_raw::binauraliserNF_destroy(std::ptr::addr_of_mut!(self.h_bin));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
