use defmt::Format;
use embassy::time::{self, Duration, Instant, Timer};
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::{peripherals, pwm, twim};

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
// TODO: Add Magnometer with some form of calibration (can maybe use lsm303agr crate)
// TODO: Add Accelerometer (can maybe use lsm303agr crate)
// TODO: Add ability to turn on leds for line sensors?
// TODO: Add ability to read ir sensor?
// TODO: Add serial, ble, or radio for communication to computer?
const BASE_ADDR: u8 = 0x47;
pub struct RobotBase<'d, T: twim::Instance, P: pwm::Instance> {
    i2c: twim::Twim<'d, T>,
    left_light_sensor: Input<'d, peripherals::P0_03>,
    right_light_sensor: Input<'d, peripherals::P0_04>,
    sonar_trig: Output<'d, peripherals::P0_13>,
    sonar_echo: Input<'d, peripherals::P1_02>,
    servo: pwm::SimplePwm<'d, P>,
}
impl<'d, T: twim::Instance, P: pwm::Instance> RobotBase<'d, T, P> {
    pub async fn new(
        i2c: twim::Twim<'d, T>,
        ll: peripherals::P0_03,
        rl: peripherals::P0_04,
        st: peripherals::P0_13,
        se: peripherals::P1_02,
        servo: peripherals::P0_01,
        pwm: P,
    ) -> Result<RobotBase<'d, T, P>, twim::Error> {
        let mut rb = RobotBase {
            i2c,
            left_light_sensor: Input::new(ll, Pull::Down),
            right_light_sensor: Input::new(rl, Pull::Down),
            sonar_trig: Output::new(st, Level::Low, OutputDrive::Standard),
            sonar_echo: Input::new(se, Pull::Down),
            servo: pwm::SimplePwm::new_1ch(pwm, servo),
        };
        // microservo requires 50hz or 20ms period
        // set_period can only set down to 125khz so we cant use it directly
        // Div128 is 125khz or 0.000008s or 0.008ms, 20/0.008 is 2500 is top
        rb.servo.set_prescaler(pwm::Prescaler::Div128);
        rb.servo.set_max_duty(2500);
        rb.servo.disable();

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

    pub fn disable_servo(&mut self) {
        self.servo.disable()
    }

    pub fn enable_servo(&mut self) {
        self.servo.enable()
    }

    pub fn servo(&mut self, angle: u8) {
        if angle > 180 {
            defmt::warn!(
                "Angle should be between 0 and 180 inclusive. Got: {}",
                angle
            );
        }
        // Servo seems to be slightly off center. Adjusting here.
        let angle = angle + 5;
        // 1ms 45deg (1/.008=125), 1.5ms 90deg (1.5/.008=187.5), 2ms 135deg (2/.008=250),
        // Angle range: about 180??(in 500???2500??sec)
        // Map value to 500 to 2500 us.
        let us = angle as u32 * 2000 / 180 + 500;
        // Divide by 0.008 (multiply by 125) and divide by 1000 to get final value in ms.
        let off_duty = us * 125 / 1000;
        let duty = 2500 - off_duty;
        defmt::debug!("Setting servo to: {}", duty);
        self.servo.set_duty(0, duty as u16)
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

    pub fn light_left(&self) -> LightLevel {
        if self.left_light_sensor.is_low() {
            LightLevel::Bright
        } else {
            LightLevel::Dark
        }
    }

    pub fn light_right(&self) -> LightLevel {
        if self.right_light_sensor.is_low() {
            LightLevel::Bright
        } else {
            LightLevel::Dark
        }
    }

    pub fn sonar_distance(&mut self) -> Option<u32> {
        const MAX_SENSOR_DELAY: u64 = 35000;
        const MAX_SENSOR_DISTANCE_CM: u64 = 300;
        // Note: 58 assumes room temperature.
        // At 0 C, it would be 60.
        const US_ROUNDTRIP_CM: u64 = 58;
        const MAX_ECHO_TIME: u64 = MAX_SENSOR_DISTANCE_CM * US_ROUNDTRIP_CM + (US_ROUNDTRIP_CM / 2);
        self.sonar_trig.set_low();
        time::block_for(Duration::from_micros(4));
        self.sonar_trig.set_high();
        time::block_for(Duration::from_micros(10));
        self.sonar_trig.set_low();

        if self.sonar_echo.is_high() {
            defmt::warn!("Sonar pulses too close together. Echo is still high.");
            return None;
        }
        let start = Instant::now();
        let timeout = start + Duration::from_micros(MAX_SENSOR_DELAY);
        while !self.sonar_echo.is_high() {
            if Instant::now() > timeout {
                defmt::warn!(
                    "Timed out while waiting to measure sonar distances: {}us",
                    start.elapsed().as_micros()
                );
                return None;
            }
        }
        let start = Instant::now();
        let timeout = start + Duration::from_micros(MAX_ECHO_TIME);
        while self.sonar_echo.is_high() {
            if Instant::now() > timeout {
                defmt::warn!(
                    "Timed out while measuring sonar distances: {}us",
                    start.elapsed().as_micros()
                );
                return None;
            }
        }
        let echo_time = start.elapsed().as_micros();

        Some(((echo_time + US_ROUNDTRIP_CM / 2) / US_ROUNDTRIP_CM) as u32)
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
