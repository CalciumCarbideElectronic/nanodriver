// #[macro_use]
extern crate ftdi_mpsse;
mod dac;
mod error;
mod global;
mod interface;
mod svc;
mod test;
mod sin;
mod log;

use actix_web::{middleware, App, HttpServer};

/// Transaction enum defines possible SPI transactions
#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
