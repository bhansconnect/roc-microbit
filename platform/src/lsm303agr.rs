use embassy_nrf::twim;

// const ACCEL_ADDR: u8 = 0b0011001;
const MAG_ADDR: u8 = 0b0011110;

const CFG_REG_A_M: u8 = 0x60;
const CFG_REG_B_M: u8 = 0x61;
// const CFG_REG_C_M: u8 = 0x62;
const STATUS_REG_M: u8 = 0x67;
const OUT_BASE_REG_M: u8 = 0x68;

pub struct MagData {
    pub x: i32,
    pub y: i32,
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
        let y = (u16::from(data[2]) | (u16::from(data[3]) << 8)) as i16;
        let z = (u16::from(data[4]) | (u16::from(data[5]) << 8)) as i16;
        // These need to be scaled by 1.5 to be converted from raw to milliGuass.
        // We also convert them from milliGauss to nanoTesla by multiplying by 100.
        // This leads to times 150.
        let scaled_x = x as i32 * 150;
        let scaled_y = y as i32 * 150;
        let scaled_z = z as i32 * 150;
        Ok(MagData {
            x: scaled_x,
            y: scaled_y,
            z: scaled_z,
        })
    }
}
