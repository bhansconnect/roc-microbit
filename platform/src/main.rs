// #![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::ffi::c_void;
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use microbit::{board::Board, hal::Timer};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
// use panic_halt as _;

#[link(name = "rocapp")]
extern "C" {
    fn roc__mainForHost_1_exposed_generic(val: u8, x: &u64);
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(c_ptr: *mut c_void, tag_id: u32) {
    panic!("Roc panicked: 0x{:0x} {}", c_ptr as usize, tag_id)
}

#[inline(never)]
fn fib_lin(n: u8, x: &mut u64) {
    let mut a = 0;
    *x = 1;
    let mut c;
    if n == 0 {
        *x = a;
        return;
    }
    for _ in 2..=n {
        c = a + *x;
        a = *x;
        *x = c;
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    // for i in 0..=92 {
    timer.delay_ms(1000_u32);
    let i = 92;
    const N: usize = 10000;
    rprintln!("Calculating fib({}) {} times in roc", i, N);
    let mut x = 0;
    for _ in 0..N {
        unsafe { roc__mainForHost_1_exposed_generic(i, &mut x) };
    }
    rprintln!("Result: {}", x);

    rprintln!("Calculating fib({}) {} times in rust linear", i, N);
    for _ in 0..N {
        fib_lin(i, &mut x);
    }
    rprintln!("Result: {}", x);
    // }
    loop {}
}
