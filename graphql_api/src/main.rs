use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, HttpServer};
use std::env;
use std::time::Instant;
use tera::Tera;
use tera_text_filters::snake_case;

use graphql_api::database::{self, POOL};
use graphql_api::graphql::create_schema_with_context;
use graphql_api::handlers;
use graphql_api::AppData;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    println!("Starting DB initialization");
    let now = Instant::now();
    database::init();
    println!("DB initialization done in {}s.", now.elapsed().as_secs());

    // One-off seeding path. Run as a separate process (e.g. `heroku run
    // "./target/release/graphql_api --seed"`) so the heavy demo-data
    // generation never blocks web dyno boot and trips Heroku's R10 timeout.
    if env::args().any(|arg| arg == "--seed") {
        println!("Running one-off database seed");
        let seed_start = Instant::now();
        database::seed();
        println!("Seed finished in {}s.", seed_start.elapsed().as_secs());
        return Ok(());
    }

    let environment = env::var("ENVIRONMENT");

    let environment = match environment {
        Ok(v) => v,
        Err(_) => String::from("test"),
    };

    let _secret_key = env::var("SECRET_KEY").expect("Unable to find secret key");

    let (host, port) = if environment == "production" {
        let p: u16 = env::var("PORT")
            .unwrap()
            .parse()
            .expect("Unable to convert string to u16");
        (env::var("HOST").unwrap(), p)
    } else {
        (String::from("0.0.0.0"), 8080)
    };

    let _domain = host.clone();

    println!("Manifests dir: {}", env!("CARGO_MANIFEST_DIR"));

    println!("Serving on: {}:{}", &host, &port);

    // Create Schema
    let schema = web::Data::new(create_schema_with_context(POOL.clone()));
    println!("Got schema");

    HttpServer::new(move || {
        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string());

        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
            .max_age(3600);

        for origin in allowed_origins.split(',') {
            cors = cors.allowed_origin(origin.trim());
        }

        let mut tera = Tera::new("graphql_api/templates/**/*").unwrap();

        tera.register_filter("snake_case", snake_case);
        tera.full_reload()
            .expect("Error running auto reload with Tera");

        let app_data = web::Data::new(AppData { tmpl: tera });

        App::new()
            .wrap(cors)
            //.data(POOL.clone())
            .configure(handlers::configure_services)
            .app_data(schema.clone())
            .app_data(app_data)
            .wrap(middleware::Logger::default())
    })
    .bind((host, port))?
    .run()
    .await
}
