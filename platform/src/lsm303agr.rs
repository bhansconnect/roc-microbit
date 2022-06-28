use defmt::Format;
use embassy::time::Instant;
use embassy_nrf::twim;

// const ACCEL_ADDR: u8 = 0b0011001;
const MAG_ADDR: u8 = 0b0011110;

const CFG_REG_A_M: u8 = 0x60;
const CFG_REG_B_M: u8 = 0x61;
// const CFG_REG_C_M: u8 = 0x62;
const STATUS_REG_M: u8 = 0x67;
const OUT_BASE_REG_M: u8 = 0x68;

#[derive(Format)]
pub struct MagData {
    pub x: i32,
    // TODO: Maybe re-add y, but it is not used for the current robot or calibrated.
    // pub y: i32,
    pub z: i32,
}

pub struct Lsm303agr<'d, T: twim::Instance> {
    i2c: twim::Twim<'d, T>,
}
impl<'d, T: twim::Instance> Lsm303agr<'d, T> {
    pub async fn new(mut i2c: twim::Twim<'d, T>) -> Result<Lsm303agr<'d, T>, twim::Error> {
        // Set to continous mode with high resolution and 100Hz ODR.
        i2c.write(MAG_ADDR, &[CFG_REG_A_M, 0b00001100]).await?;
        // Enable low pass filter.
        i2c.write(MAG_ADDR, &[CFG_REG_B_M, 0b00000001]).await?;
        // Uncalibrate the magnometer. We will offset and scale it in the code here.
        // TODO: setup accelometer.

        Ok(Lsm303agr { i2c })
    }

    pub async fn mag_ready(&mut self) -> Result<bool, twim::Error> {
        let mut data = [0];
        self.i2c
            .write_read(MAG_ADDR, &[STATUS_REG_M], &mut data)
            .await?;

        // zyx data available.
        let zyxda = 0b00001000;
        Ok(data[0] & zyxda == zyxda)
    }

    pub async fn mag_data(&mut self) -> Result<MagData, twim::Error> {
        let mut data = [0; 6];
        self.i2c
            .write_read(MAG_ADDR, &[OUT_BASE_REG_M | 0x80], &mut data)
            .await?;
        let x = (u16::from(data[0]) | (u16::from(data[1]) << 8)) as i16;
        // let y = (u16::from(data[2]) | (u16::from(data[3]) << 8)) as i16;
        let z = (u16::from(data[4]) | (u16::from(data[5]) << 8)) as i16;
        // These need to be scaled by 1.5 to be converted from raw to milliGuass.
        // We also convert them from milliGauss to nanoTesla by multiplying by 100.
        // This leads to times 150.
        let scaled_x = x as i32 * 150;
        // let scaled_y = y as i32 * 150;
        let scaled_z = z as i32 * 150;

        // Finally apply hard and soft iron calibration.
        // These were calculated with this method: https://www.appelsiini.net/2018/calibrate-magnetometer/
        // Center: (77325, -11700.0)
        // Scale: (0.9636118598382749, 1.0392441860465116)
        // Note, the USB cable definitely affects the hard iron offset...so this is probably off by a few thousand.
        // Staying in interger since the numbers are between +/-35,000
        let calibrated_x = ((scaled_x - 77325) * 09_636) / 10_000;
        let calibrated_z = ((scaled_z + 11700) * 11_700) / 10_000;
        Ok(MagData {
            x: calibrated_x,
            // y: scaled_y,
            z: calibrated_z,
        })
    }

    pub async fn mag_heading(&mut self) -> Result<(MagData, f32), twim::Error> {
        let data = self.mag_data().await?;
        let heading = libm::atan2f(data.x as f32, data.z as f32);
        Ok((data, heading))
    }
}

// Where would I add the fact that magnitude = sqrt(x*x+z*z)?
// Would this require adding x and z as state variables?
// Actually I think it may require making magnitude a measurement?

// We currently don't have any other angle sensors, so this will just estimate by itself.
// Once we have wheel encoders, it should be possible to get a pretty noisy delta angle reading.
// This is a Kalman filter that will go from x, z to the angle of heading.
pub struct MagFilter {
    // State is tracking angle, delta_angle, magnitude, x_bias, z_bias, x_scale, z_scale.
    last_t: Instant,
    x: na::SVector<f32, 7>,    // State Vector
    p: na::SMatrix<f32, 7, 7>, // Estimate Uncertainty
    q: na::SMatrix<f32, 7, 7>, // Process Noise Uncertainty
    r: na::SMatrix<f32, 2, 2>, // Measurement Uncertainty
}

impl MagFilter {
    pub fn new(mag_full: (MagData, f32)) -> MagFilter {
        let (mag_data, heading) = mag_full;
        let x = mag_data.x as f32;
        let z = mag_data.z as f32;
        let magnitude = libm::sqrtf(x * x + z * z);
        MagFilter {
            last_t: Instant::now(),
            x: na::SVector::from([heading, 0.0, magnitude, 0.0, 0.0, 1.0, 1.0]),
            // TODO: Actually setup/tune below values this.
            #[rustfmt::skip]
            p: na::SMatrix::from_row_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]),
            #[rustfmt::skip]
            q: na::SMatrix::from_row_slice(&[
                1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ]),
            #[rustfmt::skip]
            r: na::SMatrix::from_row_slice(&[
                1.0, 0.0,
                0.0, 1.0,
            ]),
        }
    }

    fn predict(&mut self) {
        // State Update:
        // angle = angle + delta_angle * dt
        // Everything else is constant
        let dt = self.last_t.elapsed();
        self.last_t = Instant::now();
        let mut f = na::SMatrix::<f32, 7, 7>::identity();
        // angle, delta_angle = dt
        f[(0, 1)] = dt.as_micros() as f32 / 1_000_000.0;

        // This function is linear so it is fine to directly apply.
        // Normally, f would be the jacobian and we would have to directly update x.
        self.x = f * self.x;
        self.p = f * self.p * f.transpose() + self.q;
    }

    fn update(&mut self, mag_full: &(MagData, f32)) -> na::SVector<f32, 7> {
        let (mag_data, heading) = mag_full;
        let angle = self.x[0];
        // let dangle = self.x[1];
        let magnitude = self.x[2];
        let x_bias = self.x[3];
        let z_bias = self.x[4];
        let x_scale = self.x[5];
        let z_scale = self.x[6];
        let sin_angle = libm::sinf(angle);
        let cos_angle = libm::cosf(angle);
        // Measurement equations:
        // x = (magnitude * sin(angle) * x_scale) + x_bias
        // z = (magnitude * cos(angle) * z_scale) + z_bias
        let hx = na::SVector::from([
            (magnitude * sin_angle * x_scale) + x_bias,
            (magnitude * cos_angle * z_scale) + z_bias,
        ]);
        // Measurement jacobian.
        #[rustfmt::skip]
        let h = na::SMatrix::<f32, 2, 7>::from_row_slice(&[
            // dx/d_angle = magnitude * cos(angle) * x_scale
            // dz/d_angle = magnitude * -1 * sin(angle) * z_scale
            magnitude * cos_angle * x_scale, magnitude * -1.0 * sin_angle * z_scale,
            // dx/d2_angle = 0
            0.0, 0.0,
            // dx/d_magnitude = sin(angle) * x_scale
            // dz/d_magnitude = cos(angle) * z_scale
            sin_angle * x_scale, cos_angle * z_scale,
            // dx/d_x_bias = 1
            1.0, 0.0,
            // dz/d_z_bias = 1
            0.0, 1.0,
            // dx/d_x_scale = magnitide * sin(angle)
            magnitude * sin_angle, 0.0,
            // dz/d_z_scale = magnitide * cos(angle)
            0.0, magnitude * cos_angle,
        ]);

        // TODO: investigate inverse, maybe use pseudo inverse?
        let k =
            self.p * h.transpose() * (h * self.p * h.transpose() + self.r).try_inverse().unwrap();
        let t = na::SMatrix::identity() - k * h;
        self.p = t * self.p * t.transpose() + k * self.r * k.transpose();
        let z = na::SVector::from([mag_data.x as f32, mag_data.z as f32]);
        self.x = self.x + k * (z - hx);
        self.x
    }

    pub fn predict_and_update(&mut self, mag_full: &(MagData, f32)) -> na::SVector<f32, 7> {
        self.predict();
        self.update(mag_full)
    }
}
