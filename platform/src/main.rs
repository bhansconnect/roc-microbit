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

#[repr(u8)]
#[derive(Format, Default, Clone)]
enum LightLevel {
    #[default]
    Bright = 0,
    Dark = 1,
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

#[repr(u8)]
#[derive(Format, Default, Clone)]
enum LightState {
    #[default]
    Off = 0,
    On = 1,
}

#[repr(u8)]
#[derive(Format, Default, Clone)]
enum Direction {
    #[default]
    Forward = 0,
    Reverse = 1,
}

// Robot Base is now KeyeStudio Microbit 4WD Mecanum Robot Kit.
const BASE_ADDR: u8 = 0x47;
struct RobotBase<'a, T: twim::Instance> {
    i2c: twim::Twim<'a, T>,
}
impl<'a, T: twim::Instance> RobotBase<'a, T> {
    async fn new(i2c: twim::Twim<'a, T>) -> Result<RobotBase<'a, T>, twim::Error> {
        let mut rb = RobotBase { i2c };
        rb.i2c.write(BASE_ADDR, &[0x00, 0x00]).await?;
        rb.set_all_pwm(0, 0).await?;
        rb.i2c.write(BASE_ADDR, &[0x01, 0x04]).await?;
        rb.i2c.write(BASE_ADDR, &[0x00, 0x01]).await?;

        Timer::after(Duration::from_millis(5)).await;

        rb.i2c.write(BASE_ADDR, &[0x00]).await?;
        let mut models = [0];
        rb.i2c.read(BASE_ADDR, &mut models).await?;
        let model = models[0] & !0x10;
        rb.i2c.write(BASE_ADDR, &[0x00, model]).await?;
        Timer::after(Duration::from_millis(5)).await;
        Ok(rb)
    }
    async fn set_pwm(&mut self, channel: u8, on: u16, off: u16) -> Result<(), twim::Error> {
        self.i2c
            .write(BASE_ADDR, &[0x06 + 4 * channel, (on & 0xFF) as u8])
            .await?;
        self.i2c
            .write(BASE_ADDR, &[0x07 + 4 * channel, (on >> 8) as u8])
            .await?;
        self.i2c
            .write(BASE_ADDR, &[0x08 + 4 * channel, (off & 0xFF) as u8])
            .await?;
        self.i2c
            .write(BASE_ADDR, &[0x09 + 4 * channel, (off >> 8) as u8])
            .await?;
        Ok(())
    }

    async fn set_all_pwm(&mut self, on: u16, off: u16) -> Result<(), twim::Error> {
        self.i2c
            .write(BASE_ADDR, &[0xFA, (on & 0xFF) as u8])
            .await?;
        self.i2c.write(BASE_ADDR, &[0xFB, (on >> 8) as u8]).await?;
        self.i2c
            .write(BASE_ADDR, &[0xFC, (off & 0xFF) as u8])
            .await?;
        self.i2c.write(BASE_ADDR, &[0xFD, (off >> 8) as u8]).await?;
        Ok(())
    }

    async fn left_led(&mut self, state: LightState) -> Result<(), twim::Error> {
        match state {
            LightState::On => self.set_pwm(12, 0, 4095).await?,
            LightState::Off => self.set_pwm(12, 0, 0).await?,
        };
        Ok(())
    }

    async fn right_led(&mut self, state: LightState) -> Result<(), twim::Error> {
        match state {
            LightState::On => self.set_pwm(13, 0, 4095).await?,
            LightState::Off => self.set_pwm(13, 0, 0).await?,
        };
        Ok(())
    }

    async fn drive_motor(
        &mut self,
        pwm0: u8,
        pwm1: u8,
        pwm2: u8,
        dir: Direction,
        speed: u16,
    ) -> Result<(), twim::Error> {
        if speed > 4095 {
            defmt::warn!(
                "Speed should be between 0 and 4095 inclusive. Got speed: {}",
                speed,
            );
        }
        match dir {
            Direction::Forward => {
                self.set_pwm(pwm0, 0, 0).await?;
                self.set_pwm(pwm1, 4096, 0).await?;
                self.set_pwm(pwm2, 0, speed).await?;
            }
            Direction::Reverse => {
                self.set_pwm(pwm0, 4096, 0).await?;
                self.set_pwm(pwm1, 0, 0).await?;
                self.set_pwm(pwm2, 0, speed).await?;
            }
        }
        Ok(())
    }

    async fn front_left_motor(&mut self, dir: Direction, speed: u16) -> Result<(), twim::Error> {
        self.drive_motor(4, 3, 5, dir, speed).await
    }

    async fn back_left_motor(&mut self, dir: Direction, speed: u16) -> Result<(), twim::Error> {
        self.drive_motor(10, 9, 11, dir, speed).await
    }

    async fn front_right_motor(&mut self, dir: Direction, speed: u16) -> Result<(), twim::Error> {
        self.drive_motor(2, 1, 0, dir, speed).await
    }

    async fn back_right_motor(&mut self, dir: Direction, speed: u16) -> Result<(), twim::Error> {
        self.drive_motor(8, 7, 6, dir, speed).await
    }
}

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) {
    let config = twim::Config::default();
    let irq = interrupt::take!(SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0);
    let i2c = twim::Twim::new(p.TWISPI0, irq, p.P1_00, p.P0_26, config);
    let mut robot_base = RobotBase::new(i2c)
        .await
        .expect("Failed to initialize robot base.");
    robot_base
        .front_left_motor(Direction::Reverse, 4096 / 4)
        .await
        .unwrap();
    robot_base
        .back_left_motor(Direction::Forward, 4096 / 4)
        .await
        .unwrap();
    robot_base
        .front_right_motor(Direction::Reverse, 4096 / 4)
        .await
        .unwrap();
    robot_base
        .back_right_motor(Direction::Forward, 4096 / 4)
        .await
        .unwrap();
    Timer::after(Duration::from_secs(5)).await;
    robot_base
        .front_left_motor(Direction::Reverse, 0)
        .await
        .unwrap();
    robot_base
        .back_left_motor(Direction::Forward, 0)
        .await
        .unwrap();
    robot_base
        .front_right_motor(Direction::Reverse, 0)
        .await
        .unwrap();
    robot_base
        .back_right_motor(Direction::Forward, 0)
        .await
        .unwrap();

    let mut disp = Display::new(
        p.P0_28, p.P0_11, p.P0_31, p.P1_05, p.P0_30, p.P0_21, p.P0_22, p.P0_15, p.P0_24, p.P0_19,
    );

    let light_left = Input::new(p.P0_17, Pull::Up);
    let light_right = Input::new(p.P0_01, Pull::Up);
    let mut input: RocInput = Default::default();
    defmt::info!("Starting Main Loop");
    loop {
        defmt::debug!("Input: {}", input);
        let output = roc_main(input.clone());
        defmt::debug!("Output: {}", output);
        disp.show(&output.display, output.delay_ms).await;

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
