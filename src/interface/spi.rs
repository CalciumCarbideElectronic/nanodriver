use std::sync::Arc;

use embedded_hal::prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex};
use ftdi_embedded_hal as hal;
use hal::{FtHal, Initialized};
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
    pub(crate) _ft: Arc<FtHal<Ft4232h, Initialized>>,
}
//

unsafe impl Send for FtdiSPIController {}
unsafe impl Sync for FtdiSPIController {}
impl Transactional for FtdiSPIController {
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), IError> {
        let mut spi: hal::Spi<_> = self._ft.spi()?;
        spi.write(prefix)?;
        for d in data.iter_mut() {
            *d = spi.read().unwrap();
        }
        Ok(())
    }

    fn spi_write(&mut self, data: &[u8]) -> Result<(), IError> {
        let mut spi: hal::Spi<_> = self._ft.spi()?;
        spi.write(data)?;
        Ok(())
    }
}
