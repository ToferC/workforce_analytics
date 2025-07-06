# Web Starter Rust

This repo is a starter app for my Web-dev. I've probably built something similar about six times, so hopefully this forestalls a 7th.

- [x] Actix-Web w/ async
- [x] Tera for templates
- [x] Diesel accessing Postgresql DB
- [x] User models
- [x] Automated Admin Generation
- [x] Authentication and sign-in
- [x] Email verification and reset password
- [x] Static files
- [x] Fluent integration for i18n

## Dependencies
* Diesel-cli

## Setup
* Clone the repository
* Create `.env` file with the following environmental variables:
    * COOKIE_SECRET_KEY=MINIMUM32CHARACTERS
    * DATABASE_URL
    * SENDGRID_API_KEY=YOUR_API_KEY
    * ADMIN_NAME="YOUR NAME"
    * ADMIN_EMAIL=your@email.com
    * ADMIN_PASSWORD=MINIMUM12CHARACTERS
    * ENVIRONMENT=test
* Change APP_NAME const in lib.rs to your app
* `diesel migration run`
* `cargo run`
