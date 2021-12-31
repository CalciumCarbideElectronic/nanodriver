use embedded_hal::digital::v2::OutputPin;
use libftd2xx::Ft4232h;

use crate::error::IError;
use ftdi_embedded_hal::OutputPin as FtOutPin;
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

// type Q = &'static Lazy<FtHal<Ft4232h, Initialized>>;
// type PinFactory = Box<dyn Fn() -> FtOutPin<'static, Ft4232h>>;
#[allow(dead_code)]
pub struct FtdiGPIOController<'b> {
    // pub(crate) _ft: Arc<FtHal<Ft4232h, Initialized>>,

    // pin: Pin,
    // pub(crate) _pin: Box<dyn for<'a> Fn(&'a FtHal<Ft4232h, Initialized>) -> FtOutPin<'a, Ft4232h>>,
    // _ft: Q,
    pub(crate) _pin: FtOutPin<'b, Ft4232h>, // _phantom: &'a PhantomData<()>,
}
unsafe impl<'a> Send for FtdiGPIOController<'a> {}
unsafe impl<'a> Sync for FtdiGPIOController<'a> {}

#[allow(dead_code)]
impl<'a> FtdiGPIOController<'a> {
    pub fn new_boxed(_pin: FtOutPin<'static, Ft4232h>) -> Box<FtdiGPIOController<'a>> {
        Box::new(Self { _pin })
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
