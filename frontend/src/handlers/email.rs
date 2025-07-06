use serde::{Deserialize};


// for for single email address
#[derive(Debug, Deserialize)]
pub struct EmailForm {
    pub email: String,
}
// form for multiple email addresses
#[derive(Debug, Deserialize)]
pub struct EmailsForm {
    pub emails: String,
}