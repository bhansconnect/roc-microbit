#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt::Format;
use embassy::executor::Spawner;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{AnyPin, Input, Level, Output, OutputDrive, Pin, Pull};
use embassy_nrf::{interrupt, peripherals, twim, Peripherals};

mod fmt;

mod memory;

#[repr(C)]
#[derive(Debug, Format, Default)]
struct Row {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
}

#[repr(C)]
#[derive(Debug, Format, Default)]
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

const CUTEBOT_ADDR: u8 = 0x10;

const DEFAULT_DELAY_MS: u64 = 2;
struct Display<'d> {
    cols: [Output<'d, AnyPin>; 5],
    rows: [Output<'d, AnyPin>; 5],
}

impl<'d> Display<'d> {
    fn new(
        p0_28: peripherals::P0_28,
        p0_11: peripherals::P0_11,
        p0_31: peripherals::P0_31,
        p1_05: peripherals::P1_05,
        p0_30: peripherals::P0_30,
        p0_21: peripherals::P0_21,
        p0_22: peripherals::P0_22,
        p0_15: peripherals::P0_15,
        p0_24: peripherals::P0_24,
        p0_19: peripherals::P0_19,
    ) -> Display<'d> {
        Display {
            cols: [
                Output::new(p0_28.degrade(), Level::High, OutputDrive::Standard),
                Output::new(p0_11.degrade(), Level::High, OutputDrive::Standard),
                Output::new(p0_31.degrade(), Level::High, OutputDrive::Standard),
                Output::new(p1_05.degrade(), Level::High, OutputDrive::Standard),
                Output::new(p0_30.degrade(), Level::High, OutputDrive::Standard),
            ],
            rows: [
                Output::new(p0_21.degrade(), Level::Low, OutputDrive::Standard),
                Output::new(p0_22.degrade(), Level::Low, OutputDrive::Standard),
                Output::new(p0_15.degrade(), Level::Low, OutputDrive::Standard),
                Output::new(p0_24.degrade(), Level::Low, OutputDrive::Standard),
                Output::new(p0_19.degrade(), Level::Low, OutputDrive::Standard),
            ],
        }
    }

    // TODO: Maybe claim a timer and make this non-blocking.
    async fn show(&mut self, data: &DisplayData, duration_ms: u64) {
        let loops = duration_ms / (5 * DEFAULT_DELAY_MS);
        let matrix = data.to_bytes();

        for _ in 0..loops {
            for (row, pixel_line) in self.rows.iter_mut().zip(matrix.iter()) {
                row.set_high();
                for (col, pixel) in self.cols.iter_mut().zip(pixel_line.iter()) {
                    // TODO: deal with dimming.
                    if *pixel > 0 {
                        col.set_low();
                    }
                }
                Timer::after(Duration::from_millis(DEFAULT_DELAY_MS)).await;
                row.set_low();
                for col in self.cols.iter_mut() {
                    col.set_high();
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Format, Default, Clone)]
enum LightLevel {
    #[default]
    Dark,
    Bright,
}

#[repr(C)]
#[derive(Debug, Format, Default, Clone)]
struct RocInput {
    state: u64,
    light_left: LightLevel,
    light_right: LightLevel,
}

#[repr(C)]
#[derive(Debug, Format, Default)]
struct RocOutput {
    state: u64,
    display: DisplayData,
}

fn roc_main(input: RocInput) -> RocOutput {
    #[link(name = "app")]
    extern "C" {
        #[link_name = "roc__mainForHost_1_exposed_generic"]
        fn call(input: RocInput, out: &mut RocOutput);
    }
    let mut out: RocOutput = Default::default();
    unsafe { call(input, &mut out) };
    out
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let config = twim::Config::default();
    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let mut i2c = twim::Twim::new(p.TWISPI0, irq, p.P1_00, p.P0_26, config);
    i2c.write(CUTEBOT_ADDR, &[0x01, 0x02, 30, 0]).await.unwrap();

    let mut disp = Display::new(
        p.P0_28, p.P0_11, p.P0_31, p.P1_05, p.P0_30, p.P0_21, p.P0_22, p.P0_15, p.P0_24, p.P0_19,
    );

    let light_left = Input::new(p.P0_17, Pull::Up);
    let light_right = Input::new(p.P0_01, Pull::Up);
    let mut input: RocInput = Default::default();
    defmt::info!("Starting");
    loop {
        defmt::debug!("Input: {:?}", input);
        let output = roc_main(input.clone());
        defmt::info!("Next state: {}", output.state);
        defmt::debug!("{:?}", output.display.to_bytes());
        disp.show(&output.display, 1000).await;
        input.state = output.state;
        input.light_left = if light_left.is_high() {
            LightLevel::Bright
        } else {
            LightLevel::Dark
        };
        input.light_right = if light_right.is_high() {
            LightLevel::Bright
        } else {
            LightLevel::Dark
        };
    }
}
