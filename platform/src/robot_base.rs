use defmt::Format;
use embassy::time::{Duration, Timer};
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::{peripherals, twim};

#[repr(u8)]
#[derive(Format, Default, Clone)]
pub enum LightLevel {
    #[default]
    Bright = 0,
    Dark = 1,
}

#[repr(u8)]
#[derive(Format, Default, Clone)]
pub enum LightState {
    #[default]
    Off = 0,
    On = 1,
}

#[repr(u8)]
#[derive(Format, Default, Clone)]
pub enum Direction {
    #[default]
    Forward = 0,
    Reverse = 1,
}

// Robot Base is now KeyeStudio Microbit 4WD Mecanum Robot Kit.
const BASE_ADDR: u8 = 0x47;
pub struct RobotBase<'d, T: twim::Instance> {
    i2c: twim::Twim<'d, T>,
    left_light_sensor: Input<'d, peripherals::P0_03>,
    right_light_sensor: Input<'d, peripherals::P0_04>,
    sonar_trig: Output<'d, peripherals::P0_13>,
    sonar_echo: Input<'d, peripherals::P1_02>,
}
impl<'d, T: twim::Instance> RobotBase<'d, T> {
    pub async fn new(
        i2c: twim::Twim<'d, T>,
        ll: peripherals::P0_03,
        rl: peripherals::P0_04,
        st: peripherals::P0_13,
        se: peripherals::P1_02,
    ) -> Result<RobotBase<'d, T>, twim::Error> {
        let mut rb = RobotBase {
            i2c,
            left_light_sensor: Input::new(ll, Pull::Down),
            right_light_sensor: Input::new(rl, Pull::Down),
            sonar_trig: Output::new(st, Level::Low, OutputDrive::Standard),
            sonar_echo: Input::new(se, Pull::Down),
        };
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

    pub async fn left_led(&mut self, state: LightState) -> Result<(), twim::Error> {
        match state {
            LightState::On => self.set_pwm(12, 0, 4095).await?,
            LightState::Off => self.set_pwm(12, 0, 0).await?,
        };
        Ok(())
    }

    pub async fn right_led(&mut self, state: LightState) -> Result<(), twim::Error> {
        match state {
            LightState::On => self.set_pwm(13, 0, 4095).await?,
            LightState::Off => self.set_pwm(13, 0, 0).await?,
        };
        Ok(())
    }

    pub fn light_left(&mut self) -> LightLevel {
        if self.left_light_sensor.is_low() {
            LightLevel::Bright
        } else {
            LightLevel::Dark
        }
    }

    pub fn light_right(&mut self) -> LightLevel {
        if self.right_light_sensor.is_low() {
            LightLevel::Bright
        } else {
            LightLevel::Dark
        }
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

    pub async fn front_left_motor(
        &mut self,
        dir: Direction,
        speed: u16,
    ) -> Result<(), twim::Error> {
        self.drive_motor(4, 3, 5, dir, speed).await
    }
    pub async fn back_left_motor(&mut self, dir: Direction, speed: u16) -> Result<(), twim::Error> {
        self.drive_motor(10, 9, 11, dir, speed).await
    }
    pub async fn front_right_motor(
        &mut self,
        dir: Direction,
        speed: u16,
    ) -> Result<(), twim::Error> {
        self.drive_motor(2, 1, 0, dir, speed).await
    }
    pub async fn back_right_motor(
        &mut self,
        dir: Direction,
        speed: u16,
    ) -> Result<(), twim::Error> {
        self.drive_motor(8, 7, 6, dir, speed).await
    }

    pub async fn stop_front_left_motor(&mut self) -> Result<(), twim::Error> {
        self.front_left_motor(Direction::Forward, 0).await
    }
    pub async fn stop_back_left_motor(&mut self) -> Result<(), twim::Error> {
        self.back_left_motor(Direction::Forward, 0).await
    }
    pub async fn stop_front_right_motor(&mut self) -> Result<(), twim::Error> {
        self.front_right_motor(Direction::Forward, 0).await
    }
    pub async fn stop_back_right_motor(&mut self) -> Result<(), twim::Error> {
        self.back_right_motor(Direction::Forward, 0).await
    }
}
