#![no_main]
#![no_std]

use cortex_m_rt::entry;
// use embedded_hal::blocking::delay::DelayMs;
use microbit::{board::Board, display::blocking::Display, hal::Timer};
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
// use panic_halt as _;

mod memory;

#[repr(C)]
#[derive(Debug, Default)]
struct Row {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
}

#[repr(C)]
#[derive(Debug, Default)]
struct DisplayData {
    a: Row,
    b: Row,
    c: Row,
    d: Row,
    e: Row,
}

impl DisplayData {
    fn to_bytes(&self) -> [[u8; 5]; 5] {
        [
            [self.a.a, self.a.b, self.a.c, self.a.d, self.a.e],
            [self.b.a, self.b.b, self.b.c, self.b.d, self.b.e],
            [self.c.a, self.c.b, self.c.c, self.c.d, self.c.e],
            [self.d.a, self.d.b, self.d.c, self.d.d, self.d.e],
            [self.e.a, self.e.b, self.e.c, self.e.d, self.e.e],
        ]
    }
}

#[repr(C)]
#[derive(Debug, Default)]
struct Output {
    // TODO: move this out of here and make it a proper boxed model.
    // Currently for simplicity, the state is just a u64.
    next: u64,
    display: DisplayData,
}

fn roc_main(i: u64) -> Output {
    #[link(name = "app")]
    extern "C" {
        #[link_name = "roc__mainForHost_1_exposed_generic"]
        fn call(i: u64, out: &mut Output);
    }
    let mut out: Output = Default::default();
    unsafe { call(i, &mut out) };
    out
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let mut i = 0;
    loop {
        rprintln!("Sending state: {:?}", i);
        let output = roc_main(i);
        rprintln!("Roc generated: {:?}", output);
        display.show(&mut timer, output.display.to_bytes(), 10);
        i = output.next;
    }
}
