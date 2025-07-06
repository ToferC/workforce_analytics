
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct CustomError {
    pub error_status_code: u16,
    pub error_message: String,
}

impl CustomError {
    pub fn new(error_status_code: u16, error_message: String) -> CustomError {
        CustomError {
            error_status_code,
            error_message,
        }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&format!("{} = {}", self.error_message, self.error_status_code))
    }
}

// Custom Error Codes
// 404 page not found

impl ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        let status_code = match StatusCode::from_u16(self.error_status_code) {
            Ok(status_code) => status_code,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        match status_code.as_u16() {
            406 => {
                return HttpResponse::Found().header("Location","/not_authorized").finish()
            },
            408 => {
                return HttpResponse::Found().header("Location","/not_found").finish()
            },
            409 => {
                return HttpResponse::Found().header("Location","/database_error").finish()
            },
            i if i > 500 => {
                return HttpResponse::Found().header("Location","/internal_server_error").finish()
            },
            _ => return HttpResponse::Found().header("Location","/internal_server_error").finish()
        };
    }
}
