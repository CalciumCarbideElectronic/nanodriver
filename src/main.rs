use std::sync::Arc;

use hal::{FtHal, Initialized};

use interface::spi::FtdiSPIController;

// #[macro_use]
extern crate ftdi_mpsse;
mod dac;
mod error;
mod interface;
mod svc;
use ftdi_embedded_hal as hal;
use libftd2xx::Ft4232h;

use crate::interface::gpio::FtdiGPIOController;
use actix_web::{middleware, App, HttpServer};

/// Transaction enum defines possible SPI transactions
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ftdi: FtHal<Ft4232h, Initialized> = hal::Ft4232hHal::new()
        .expect("Failed to open FT232H device")
        .init_default()
        .expect("Failed to initialize MPSSE");

    let h = Arc::new(ftdi);
    let _spi = Box::new(FtdiSPIController { _ft: h.clone() });
    let mut _busy = FtdiGPIOController::new_boxed(&h, Box::new(|h| h.ad4()));
    let mut _ldac = FtdiGPIOController::new_boxed(&h, Box::new(|h| h.ad5()));
    let mut _reset = FtdiGPIOController::new_boxed(&h, Box::new(|h| h.ad6()));
    let mut _clr = FtdiGPIOController::new_boxed(&h, Box::new(|h| h.ad7()));

    // let state = Arc::new(AD5370 {
    //     vref: 4.0,
    //     reg: Register::default(),
    //     spi,
    //     _busy,
    //     _ldac,
    //     _reset,
    //     _clr,
    // });

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            // .data(state.clone())
            .service(svc::ping)
            .service(svc::voltage)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
    // todo!()
}
