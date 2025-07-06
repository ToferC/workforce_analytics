use actix_web::web;
use actix_web::{HttpServer, App, middleware};
use dotenv::dotenv;
use std::env;
use tera::{Tera};
use tera_text_filters::snake_case;
use actix_identity::{IdentityService, CookieIdentityPolicy};
use actix_web_static_files;

use frontend::handlers;
use frontend::AppData;

use fluent_templates::{FluentLoader, static_loader};
// https://lib.rs/crates/fluent-templates

// Setup for serving static files
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

static_loader! {
    static LOCALES = {
        locales: "./i18n/",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::init();

    let environment = env::var("ENVIRONMENT");

    let environment = match environment {
        Ok(v) => v,
        Err(_) => String::from("test"),
    };

    let (host, port) = if environment == "production" {
        (env::var("HOST").unwrap(), env::var("PORT").unwrap())
    } else {
        (String::from("127.0.0.1"), String::from("8088"))
    };

    let (api_host, api_port) = if environment == "production" {
        (env::var("GRAPHQL_API_HOST").unwrap(), env::var("GRAPHQL_API_PORT").unwrap())
    } else {
        (String::from("127.0.0.1"), String::from("8080"))
    };

    let cookie_secret_key = env::var("COOKIE_SECRET_KEY").expect("Unable to find cookie secret key");

    let mut tera = Tera::new(
        "frontend/templates/**/*").unwrap();

    tera.register_filter("snake_case", snake_case);
    tera.full_reload().expect("Error running auto-reload with Tera");
    tera.register_function("fluent", FluentLoader::new(&*LOCALES));

    let api_url = format!("http://{}:{}/graphql", api_host, api_port);
    
    println!("Serving on http://{}:{}", &host, &port);
    println!("Targeting API on {}", &api_url);
    
    
    let data = web::Data::new(AppData {
        tmpl: tera,
        api_url: api_url,
    });

    HttpServer::new(move || {
        let generated = generate();

        App::new()
            .wrap(middleware::Logger::default())
            .configure(handlers::configure_services)
            .app_data(data.clone())
            .service(actix_web_static_files::ResourceFiles::new(
                "/static", generated,
            ))
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&cookie_secret_key.as_bytes())
                .name("user-auth")
                .secure(false)))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
