#[cfg(test)]
mod e2etest {

    use crate::global::HANDLE;
    use embedded_hal::{
        digital::v2::OutputPin, prelude::_embedded_hal_spi_FullDuplex, spi::Polarity,
    };

    use std::{thread::sleep, time::Duration};

    use crate::dac::ad537x::reg::ChannelAddress;

    use crate::global::{FTDI, GLOBAL_AD5370};

    #[test]
    fn test_spi_pattern() {
        let mut spi = FTDI.spi().unwrap();
        let mut cs = FTDI.ad3();
        spi.set_clock_polarity(Polarity::IdleLow);
        for _ in 0..100 {
            cs.set_low().unwrap();
            spi.send(0x42).unwrap();
            spi.send(0xA3).unwrap();
            spi.send(0x57).unwrap();
            cs.set_high().unwrap();
        }
    }

    #[test]
    fn test_set_voltage() {
        let mut guard = GLOBAL_AD5370.lock().unwrap();
        guard.init().unwrap();
        for _i in 0..0xFFFF {
            guard._ldac.set().unwrap();
            guard
                .set_code(_i, crate::dac::ad537x::reg::ChannelAddress::AllCh)
                .unwrap();
            guard._ldac.reset().unwrap();
        }
    }

    #[test]
    fn test_set_code() {
        let mut guard = GLOBAL_AD5370.lock().unwrap();
        guard.set_gain(0xF000).unwrap();
        guard.set_offset(0x8000).unwrap();
        guard._ldac.set().unwrap();
        // guard._ldac.set().unwrap();
        guard
            .set_code(0xFFFF, crate::dac::ad537x::reg::ChannelAddress::AllCh)
            .unwrap();
        sleep(Duration::from_millis(200));
        guard._ldac.reset().unwrap();

        // guard._clr.set().unwrap();
        // guard._ldac.reset().unwrap();
        // guard._ldac.set().unwrap();
    }

    #[test]
    fn test_ldac() {
        let mut guard = GLOBAL_AD5370.lock().unwrap();
        guard._ldac.reset().unwrap();
        guard._ldac.set().unwrap();
    }

    #[test]
    fn test_sin() {
        unsafe {
            if let Some(h) = HANDLE.take() {
                h.join();
            }
        }
    }

    #[test]
    fn test_read_all() {
        let mut guard = GLOBAL_AD5370.lock().unwrap();
        guard.init().unwrap();
        guard.set_voltage(0.1, ChannelAddress::AllCh).unwrap();
        guard.set_offset(0x1000).unwrap();
        guard.read_all().unwrap();
    }
}
