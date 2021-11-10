use ftdi_embedded_hal::OutputPin as FtOutPin;

use embedded_hal::{
    digital::v2::OutputPin,
    prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex},
};
use ftdi_embedded_hal as hal;
use libftd2xx::Ft4232h;

use crate::error::IError;

pub trait Transactional: Send + Sync {
    /// Read writes the prefix buffer then reads into the input buffer
    /// Note that the values of the input buffer will also be output, because, SPI...
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), IError>;

    /// Write writes the prefix buffer then writes the output buffer
    fn spi_write(&mut self, data: &[u8]) -> Result<(), IError>;
}

pub struct FtdiSPIController {
    pub(crate) _spi: hal::Spi<'static, Ft4232h>,
    pub(crate) _cs: FtOutPin<'static, Ft4232h>,
}
//

unsafe impl Send for FtdiSPIController {}
unsafe impl Sync for FtdiSPIController {}
impl Transactional for FtdiSPIController {
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), IError> {
        self._cs.set_low().unwrap();

        self._spi.write(prefix)?;

        self._cs.set_high().unwrap();

        //AD5370 t21 guard

        self._cs.set_low().unwrap();

        for d in data.iter_mut() {
            *d = self._spi.read().unwrap();
        }

        self._cs.set_high().unwrap();
        Ok(())
    }

    fn spi_write(&mut self, data: &[u8]) -> Result<(), IError> {
        self._cs.set_low().unwrap();
        self._spi.write(data)?;
        self._cs.set_high().unwrap();
        Ok(())
    }
}
