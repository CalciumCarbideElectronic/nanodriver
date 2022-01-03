use std::{convert::Infallible, error::Error, fmt::Display};

use actix_web::ResponseError;
use libftd2xx::TimeoutError;

#[derive(Debug)]
pub enum IError {
    #[allow(dead_code)]
    General {
        msg: &'static str,
    },
    Timeout {
        source: &'static str,
    },
}

impl Error for IError {}

impl Display for IError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General { msg } => f.write_str(msg),
            IError::Timeout { source } => write!(f, "timeout! src:{}", source),
        }
    }
}

impl From<TimeoutError> for IError {
    fn from(_: TimeoutError) -> Self {
        Self::Timeout {
            source: "libftd2xx",
        }
    }
}

impl ResponseError for IError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<Infallible> for IError {
    fn from(_: Infallible) -> Self {
        Self::General {
            msg: "Infallible error",
        }
    }
}

impl From<()> for IError {
    fn from(_: ()) -> Self {
        Self::General {
            msg: "unknown error",
        }
    }
}

impl From<rppal::spi::Error> for IError {
    fn from(_: rppal::spi::Error) -> Self {
        Self::General {
            msg: "rapspberry pi spi error",
        }
    }
}
