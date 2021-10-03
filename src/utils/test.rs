use std::{thread::sleep, time::Duration};

use embedded_hal::{
    digital::v2::OutputPin,
    prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_spi_FullDuplex},
    spi::Polarity,
};
use ftdi_embedded_hal as hal;
use hal::{FtHal, Initialized};
use libftd2xx::{Ft4232h, FtdiMpsse, MpsseSettings};
const SLEEP_DURATION: Duration = Duration::from_millis(2000);

fn blink(ft: FtHal<Ft4232h, Initialized>) {
    let NUM_BLINK = 100;
    let mut output_pin = ft.ad0();

    println!("Starting blinky example");
    for n in 0..NUM_BLINK {
        output_pin.set_high().expect("failed to set GPIO");
        sleep(SLEEP_DURATION);
        output_pin.set_low().expect("failed to set GPIO");
        sleep(SLEEP_DURATION);
        println!("Blinked {}/{} times", n + 1, NUM_BLINK);
    }
}

// fn spi_test(ft: &FtHal<Ft4232h, Initialized>) {
// }
fn main() {
    let settings = MpsseSettings {
        reset: true,
        in_transfer_size: 4096,
        read_timeout: Duration::from_secs(1),
        write_timeout: Duration::from_secs(1),
        latency_timer: Duration::from_millis(16),
        mask: 0,
        clock_frequency: Some(10_000),
    };
    let ftdi = hal::Ft4232hHal::new()
        .expect("Failed to open FT232H device")
        .init(&settings)
        .expect("Failed to initialize MPSSE");

    let mut spi = ftdi.spi().unwrap();
    let mut cs = ftdi.ad3();
    spi.set_clock_polarity(Polarity::IdleLow);
    for _ in 0..100 {
        // spi_test(&ftdi);

        cs.set_low();
        spi.send(0x42);
        spi.send(0xA3);
        spi.send(0x57);
        cs.set_high();
        sleep(SLEEP_DURATION);
    }

    // blink(ftdi);
}
