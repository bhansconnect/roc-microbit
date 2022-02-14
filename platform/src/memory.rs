use core::ffi::c_void;
use rtt_target::{rprint, rprintln};

// TODO: Maybe add stack based alloc as a feature.
#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, _alignment: u32) -> *mut c_void {
    panic!("allocations are not allowed for this platform")
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    c_ptr: *mut c_void,
    new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    panic!("allocations are not allowed for this platform")
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(c_ptr: *mut c_void, _alignment: u32) {
    panic!("allocations are not allowed for this platform")
}

/// # Safety
///
/// TODO: Add safety documentation.
#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    match tag_id {
        0 => {
            let mut c_str = c_ptr as *const u8;
            rprint!("Roc hit a panic: ");
            while *c_str != 0 {
                rprint!("{}", *c_str as char);
                c_str = c_str.add(1);
            }
            rprintln!("");
            panic!();
        }
        _ => panic!("Roc panicked: 0x{:0x} {}", c_ptr as usize, tag_id),
    }
}

#[no_mangle]
pub unsafe extern "C" fn roc_memcpy(dst: *mut c_void, src: *mut c_void, n: usize) -> *mut c_void {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    todo!()
}
