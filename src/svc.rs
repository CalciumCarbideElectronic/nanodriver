use actix_web::HttpResponse;

use actix_web::{
    post,
    web::{self},
    Result,
};
use serde::{Deserialize, Serialize};

use std::sync::Mutex;

use crate::dac::ad537x::Instance;

#[post("/ping")]
pub async fn ping() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/plain")
        .body("ok".to_string()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetVoltageReq {
    voltage: u16,
    group: i16,
    channel: i16,
    offset: Option<u16>,
    gain: Option<u16>,
}

#[post("/voltage")]
pub async fn voltage(
    ins: web::Data<Mutex<Instance<'_>>>,
    req: web::Json<SetVoltageReq>,
) -> Result<String, crate::error::IError> {
    // let mut m = ins.reset();
    ins.lock().unwrap().reset()?;

    return Ok(format!("{:?}", req));
}
