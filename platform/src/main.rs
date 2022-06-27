#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]

use defmt::Format;
use embassy::executor::Spawner;
use embassy::time::{Duration, Instant, Timer};
use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};
use embassy_nrf::{interrupt, peripherals, twim, Peripherals};

mod fmt;
mod lsm303agr;
mod memory;
pub mod robot_base;

use robot_base::{Direction, LightLevel, RobotBase};

#[repr(C)]
#[derive(Default)]
struct Row {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
}

#[repr(C)]
#[derive(Default)]
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

impl defmt::Format for DisplayData {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "{:?}", self.to_bytes())
    }
}

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
#[derive(Format, Default, Clone)]
struct RocInput {
    state: u64,
    light_left: LightLevel,
    light_right: LightLevel,
}

#[repr(C)]
#[derive(Format, Default)]
struct RocOutput {
    delay_ms: u64,
    state: u64,
    display: DisplayData,
    speed_left: i8,
    speed_right: i8,
}

fn roc_main(input: RocInput) -> RocOutput {
    #[link(name = "app")]
    extern "C" {
        #[link_name = "roc__mainForHost_1_exposed_generic"]
        fn call(state: u64, light_left: LightLevel, light_right: LightLevel, out: &mut RocOutput);
    }
    let mut out: RocOutput = Default::default();
    unsafe { call(input.state, input.light_left, input.light_right, &mut out) };
    out
}

// So I think the goal is going to be multithreaded sensor reading and output.
// This way, we can both display images continuously on the display and read sonar/lidar with decent accuracy.
// Of course, some of that can be offloaded to sensors that just continously scan for us.
#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let irq0 = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let i2c0 = twim::Twim::new(p.TWISPI0, irq0, p.P0_16, p.P0_08, twim::Config::default());
    let mut imu = lsm303agr::Lsm303agr::new(i2c0).await.unwrap();

    let irq1 = interrupt::take!(SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1);
    let i2c1 = twim::Twim::new(p.TWISPI1, irq1, p.P1_00, p.P0_26, twim::Config::default());
    let mut robot_base = RobotBase::new(i2c1, p.P0_03, p.P0_04, p.P0_13, p.P1_02, p.P0_01, p.PWM0)
        .await
        .expect("Failed to initialize robot base.");
    robot_base.enable_servo();
    robot_base.servo(90);
    Timer::after(Duration::from_secs(2)).await;
    robot_base.disable_servo();

    let mut disp = Display::new(
        p.P0_28, p.P0_11, p.P0_31, p.P1_05, p.P0_30, p.P0_21, p.P0_22, p.P0_15, p.P0_24, p.P0_19,
    );

    let mut input: RocInput = Default::default();
    defmt::info!("Starting Main Loop");
    let mut start = Instant::now();
    loop {
        if imu.mag_ready().await.unwrap() {
            let data = imu.mag_data().await.unwrap();
            let elapsed = start.elapsed();
            start = Instant::now();
            defmt::info!(
                "Data: ({}, {}, {}) with delay: {}us",
                data.x,
                data.y,
                data.z,
                elapsed.as_micros()
            );
        }
    }
    // loop {
    //     defmt::debug!("Sonar Distance: {:?}", robot_base.sonar_distance());
    //     // defmt::debug!("Input: {}", input);
    //     let output = roc_main(input.clone());
    //     // defmt::debug!("Output: {}", output);
    //     disp.show(&output.display, output.delay_ms).await;

    //     robot_base
    //         .front_left_motor(Direction::Forward, output.speed_left as u16 * 40)
    //         .await
    //         .unwrap();
    //     robot_base
    //         .back_left_motor(Direction::Forward, output.speed_left as u16 * 40)
    //         .await
    //         .unwrap();
    //     robot_base
    //         .front_right_motor(Direction::Forward, output.speed_right as u16 * 40)
    //         .await
    //         .unwrap();
    //     robot_base
    //         .back_right_motor(Direction::Forward, output.speed_right as u16 * 40)
    //         .await
    //         .unwrap();

    //     input.state = output.state;
    //     input.light_left = robot_base.light_left();
    //     input.light_right = robot_base.light_right();
    // }
}
