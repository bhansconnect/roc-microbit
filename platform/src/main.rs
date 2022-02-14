// #![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::ffi::c_void;
use cortex_m_rt::entry;
use embedded_hal::blocking::delay::DelayMs;
use microbit::{board::Board, hal::Timer};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod memory;
// use panic_halt as _;

pub fn roc_fib(n: u8) -> u64 {
    #[link(name = "app")]
    extern "C" {
        #[link_name = "roc__mainForHost_1_exposed_generic"]
        fn call(n: u8, out: &mut u64);
    }
    let mut out = 0;
    unsafe { call(n, &mut out) };
    out
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
    let i = 93;
    const N: usize = 10000;
    rprintln!("Calculating fib({}) {} times in roc", i, N);
    let mut x = 0;
    for _ in 0..N {
        unsafe { x = roc_fib(i) };
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
