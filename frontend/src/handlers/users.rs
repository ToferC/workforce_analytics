// example auth: https://github.com/actix/actix-extras/blob/master/actix-identity/src/lib.rs

use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, ResponseError};
use actix_identity::{Identity};
use inflector::Inflector;
use serde::{Deserialize};

use crate::{AppData, extract_identity_data, generate_basic_context};
use crate::handlers::DeleteForm;
use crate::errors::CustomError;

#[derive(Deserialize, Debug)]
pub struct UserForm {
    user_name: String,
    email: String,
}

#[derive(Deserialize, Debug)]
pub struct AdminUserForm {
    user_name: String,
    email: String,
    role: String,
    validated: String,
}

