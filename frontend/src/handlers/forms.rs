use serde::{Deserialize, Serialize};

use crate::graphql::capability_by_name_and_level::CapabilityLevel;

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct RegisterForm {
    user_name: String,
    email: String,
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct VerifyForm {
    code: String,
}

#[derive(Deserialize, Debug)]
pub struct PasswordForm {
    password: String,
}

#[derive(Deserialize, Debug)]
pub struct DeleteForm {
    pub verify: String,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CapabilityForm {
    pub name: String,
    pub level: String,
}