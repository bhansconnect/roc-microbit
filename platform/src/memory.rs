use core::ffi::c_void;

// TODO: Maybe add stack based alloc as a feature.
#[no_mangle]
pub unsafe extern "C" fn roc_alloc(_size: usize, _alignment: u32) -> *mut c_void {
    defmt::panic!("allocations are not allowed for this platform")
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    _c_ptr: *mut c_void,
    _new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    defmt::panic!("allocations are not allowed for this platform")
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(_c_ptr: *mut c_void, _alignment: u32) {
    defmt::panic!("allocations are not allowed for this platform")
}

struct RocPanic {
    c_ptr: *const u8,
}

impl defmt::Format for RocPanic {
    fn format(&self, f: defmt::Formatter) {
        let mut tmp = self.c_ptr;
        unsafe {
            while *tmp != 0 {
                defmt::write!(f, "{}", *tmp as char);
                tmp = tmp.add(1);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    match tag_id {
        0 => {
            defmt::panic!(
                "Roc hit a panic: {}",
                RocPanic {
                    c_ptr: c_ptr as *const u8
                }
            );
        }
        _ => defmt::panic!("Roc panicked: 0x{:x} {}", c_ptr as usize, tag_id),
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_memcpy(
    _dst: *mut c_void,
    _src: *mut c_void,
    _n: usize,
) -> *mut c_void {
    defmt::todo!()
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(_dst: *mut c_void, _c: i32, _n: usize) -> *mut c_void {
    defmt::todo!()
}
