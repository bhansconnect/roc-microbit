#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::Peripherals;

mod fmt;

mod memory;

// #[repr(C)]
// #[derive(Debug, Default)]
// struct Row {
//     a: u8,
//     b: u8,
//     c: u8,
//     d: u8,
//     e: u8,
// }

// #[repr(C)]
// #[derive(Debug, Default)]
// struct DisplayData {
//     a: Row,
//     b: Row,
//     c: Row,
//     d: Row,
//     e: Row,
// }

// impl DisplayData {
//     fn to_bytes(&self) -> [[u8; 5]; 5] {
//         [
//             [self.a.a, self.a.b, self.a.c, self.a.d, self.a.e],
//             [self.b.a, self.b.b, self.b.c, self.b.d, self.b.e],
//             [self.c.a, self.c.b, self.c.c, self.c.d, self.c.e],
//             [self.d.a, self.d.b, self.d.c, self.d.d, self.d.e],
//             [self.e.a, self.e.b, self.e.c, self.e.d, self.e.e],
//         ]
//     }
// }

// #[repr(C)]
// #[derive(Debug, Default)]
// struct Output {
//     // TODO: move this out of here and make it a proper boxed model.
//     // Currently for simplicity, the state is just a u64.
//     next: u64,
//     display: DisplayData,
// }

// fn roc_main(i: u64) -> Output {
//     #[link(name = "app")]
//     extern "C" {
//         #[link_name = "roc__mainForHost_1_exposed_generic"]
//         fn call(i: u64, out: &mut Output);
//     }
//     let mut out: Output = Default::default();
//     unsafe { call(i, &mut out) };
//     out
// }

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let mut col = Output::new(p.P0_28, Level::High, OutputDrive::Standard);
    let mut row = Output::new(p.P0_21, Level::Low, OutputDrive::Standard);

    col.set_low();
    loop {
        defmt::warn!("High");
        row.set_high();
        Timer::after(Duration::from_millis(1000)).await;
        defmt::info!("Low");
        row.set_low();
        Timer::after(Duration::from_millis(1000)).await;
    }
}
