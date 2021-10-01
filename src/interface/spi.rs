use std::sync::Arc;

use embedded_hal::prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex};
use ftdi_embedded_hal as hal;
use hal::{FtHal, Initialized};
use libftd2xx::{Ft4232h, TimeoutError};

use crate::error::IError;

#[derive(Debug, PartialEq)]
pub enum Transaction<'a> {
    // Write the supplied buffer to the peripheral
    Write(&'a [u8]),
    // Read from the peripheral into the supplied buffer
    Read(&'a mut [u8]),
    // Write the first buffer while reading into the second
    // This behaviour is actually just the same as Read
    //Transfer((&'a [u8], &'a mut [u8]))
}
pub trait Transactional {
    /// Read writes the prefix buffer then reads into the input buffer
    /// Note that the values of the input buffer will also be output, because, SPI...
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), IError>;

    /// Write writes the prefix buffer then writes the output buffer
    fn spi_write(&mut self, data: &[u8]) -> Result<(), IError>;
}

pub struct FtdiSPIController {
    _ft: Box<Arc<FtHal<Ft4232h, Initialized>>>,
}
//

impl Transactional for FtdiSPIController {
    fn spi_read(&mut self, prefix: &[u8], data: &mut [u8]) -> Result<(), IError> {
        let mut spi: hal::Spi<_> = self._ft.spi()?;
        spi.write(prefix)?;
        for i in 0..data.len() {
            data[i] = spi.read().unwrap();
        }
        Ok(())
    }

    fn spi_write(&mut self, data: &[u8]) -> Result<(), IError> {
        let mut spi: hal::Spi<_> = self._ft.spi()?;
        spi.write(data)?;
        Ok(())
    }
}
