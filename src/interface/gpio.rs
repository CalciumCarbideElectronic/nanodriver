use embedded_hal::digital::v2::OutputPin;
use libftd2xx::Ft4232h;

use crate::error::IError;
use ftdi_embedded_hal::OutputPin as FTOutPin;

pub trait IOController {
    fn set(&mut self) -> Result<(), IError>;
    fn reset(&mut self) -> Result<(), IError>;
    // fn read(&mut self) -> Result<bool, IError>;
}

pub struct FtdiGPIOController<'a> {
    pin: FTOutPin<'a, Ft4232h>,
}

impl<'a> FtdiGPIOController<'a> {
    #[allow(dead_code)]

    fn new(pin: FTOutPin<'a, Ft4232h>) -> FtdiGPIOController<'a> {
        Self { pin }
    }
}

impl<'a> IOController for FtdiGPIOController<'a> {
    fn set(&mut self) -> Result<(), IError> {
        self.pin.set_high()?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), IError> {
        self.pin.set_low()?;
        Ok(())
    }

    // fn read(&mut self) -> Result<bool, IError> {
    // 	self.pin.
    // }
}
