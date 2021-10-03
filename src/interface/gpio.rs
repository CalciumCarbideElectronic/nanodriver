use std::sync::Arc;

use embedded_hal::digital::v2::OutputPin;
use libftd2xx::Ft4232h;

use crate::error::IError;
use ftdi_embedded_hal::{FtHal, Initialized, OutputPin as FtOutPin};

pub trait IOController {
    fn set(&mut self) -> Result<(), IError>;
    fn reset(&mut self) -> Result<(), IError>;
    // fn read(&mut self) -> Result<bool, IError>;
}

#[allow(dead_code)]
pub enum Pin {
    AD0,
    AD1,
    AD2,
    AD3,
    AD4,
    AD5,
    AD6,
    AD7,
}

#[allow(dead_code)]
pub struct FtdiGPIOController {
    pub(crate) _ft: Arc<FtHal<Ft4232h, Initialized>>,
    pin: Pin,
}

impl FtdiGPIOController {
    // #[allow(dead_code)]

    pub fn new(_ft: Arc<FtHal<Ft4232h, Initialized>>, pin: Pin) -> FtdiGPIOController {
        Self { _ft, pin }
    }
    fn _pin(&self) -> FtOutPin<Ft4232h> {
        match self.pin {
            Pin::AD0 => self._ft.ad0(),
            Pin::AD1 => self._ft.ad1(),
            Pin::AD2 => self._ft.ad2(),
            Pin::AD3 => self._ft.ad2(),
            Pin::AD4 => self._ft.ad2(),
            Pin::AD5 => self._ft.ad2(),
            Pin::AD6 => self._ft.ad2(),
            Pin::AD7 => self._ft.ad2(),
        }
    }
}

impl IOController for FtdiGPIOController {
    fn set(&mut self) -> Result<(), IError> {
        self._pin().set_high()?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), IError> {
        self._pin().set_low()?;
        Ok(())
    }

    // fn read(&mut self) -> Result<bool, IError> {
    // 	self.pin.
    // }
}
