pub mod models;
pub mod handlers;
pub mod graphql;
pub mod errors;

use std::sync::Mutex;
use actix_web::web::Data;
use actix_web::{Error, App};
use tera::{Tera, Context};
use actix_identity::Identity;
use actix_session::Session;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

extern crate strum;
#[macro_use]
extern crate strum_macros;

const APP_NAME: &str = "Epifront";

#[derive(Clone, Debug)]
pub struct AppData {
    pub tmpl: Tera,
    pub api_url: String,
}

/// Generate context, session_user, role and node_names from id and lang
pub fn generate_basic_context(
    id: Identity,
    lang: &str,
    path: &str,
) -> (Context, String, String, String) 
{    
    let mut ctx = Context::new();

    let identity = match id.identity() {
        Some(s) => s.to_string(),
        None => "".to_string(),
    };

    // Get session data and add to context
    let (session_user, role) = (identity, "");
    ctx.insert("session_user", &session_user);
    ctx.insert("role", &role);

    let validated_lang = match lang {
        "fr" => "fr",
        "en" => "en",
        _ => "en",
    };

    ctx.insert("lang", &validated_lang);
    ctx.insert("path", &path);

    (ctx, session_user.to_owned(), role.to_owned(), lang.to_owned())
}

pub fn extract_session_data(session: &Session) -> (String, String) {

    let role_data = session.get::<String>("role").expect("Unable to get role from cookie");

    let role = match role_data {
        Some(r) => r,
        None => "".to_string(),
    };

    let user_data = session.get::<String>("user_name").expect("Unable to get user_name from cookie");

    let session_user = match user_data {
        Some(u) => u,
        None => "".to_string(),
    };

    println!("{}-{}", &session_user, &role);

    (session_user, role)
}

pub fn extract_identity_data(id: &Identity) -> Result<(), Error> {

    let id_data = id.identity();

    let session_user = match id_data {
        Some(u) => u,
        None => "".to_string(),
    };

    /*
    let user = models::User::find_slim_from_slug(&session_user);

    let role = match user {
        Ok(u) => u.role,
        _ => "".to_string()
    };

    println!("{}-{}", &session_user, &role);
    (session_user, role)
     */

    Ok(())

}

