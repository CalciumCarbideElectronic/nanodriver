use std::sync::Arc;

use embedded_hal::digital::v2::OutputPin;
use libftd2xx::Ft4232h;

use crate::error::IError;
use ftdi_embedded_hal::{FtHal, Initialized, OutputPin as FtOutPin};
pub trait IOController: Send + Sync {
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
pub struct FtdiGPIOController<'b> {
    // pub(crate) _ft: Arc<FtHal<Ft4232h, Initialized>>,

    // pin: Pin,
    // pub(crate) _pin: Box<dyn for<'a> Fn(&'a FtHal<Ft4232h, Initialized>) -> FtOutPin<'a, Ft4232h>>,
    pub(crate) _pin: FtOutPin<'b, Ft4232h>, // _phantom: &'a PhantomData<()>,
}
unsafe impl<'a> Send for FtdiGPIOController<'a> {}
unsafe impl<'a> Sync for FtdiGPIOController<'a> {}

type PinFactory = Box<dyn for<'b> Fn(&'b FtHal<Ft4232h, Initialized>) -> FtOutPin<'b, Ft4232h>>;
impl<'a> FtdiGPIOController<'a> {
    pub fn new(
        _ft: &'a Arc<FtHal<Ft4232h, Initialized>>,
        _pin_fn: PinFactory,
    ) -> FtdiGPIOController<'a> {
        let _pin = _pin_fn(_ft);
        Self { _pin }
    }
    pub fn new_boxed(
        _ft: &'a Arc<FtHal<Ft4232h, Initialized>>,
        _pin: PinFactory,
    ) -> Box<FtdiGPIOController<'a>> {
        Box::new(Self::new(_ft, _pin))
    }
}
impl<'a> FtdiGPIOController<'a> {}

impl<'a> IOController for FtdiGPIOController<'a> {
    fn set(&mut self) -> Result<(), IError> {
        self._pin.set_high()?;
        Ok(())
    }

    fn reset(&mut self) -> Result<(), IError> {
        self._pin.set_low()?;
        Ok(())
    }
}
