//! One-off database seeding command.
//!
//! Generates the demo data (skills, org structure, people, capabilities,
//! validations, publications, tasks, products). This is intentionally a
//! separate binary from the web server so the heavy seed never runs during
//! web dyno boot and trips Heroku's R10 boot timeout.
//!
//! Run as a one-off process, e.g. on Heroku:
//!
//!     heroku run "./target/release/seed" -a <api-app>
//!
//! It is guarded to no-op if the database already contains seed data, so it is
//! safe to re-run. Reset the database first if you want to re-seed from scratch.

use std::time::Instant;

use graphql_api::database;

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    // Seeding is strictly sequential, so a couple of connections is plenty.
    // The web process may already be holding most of the role's connection
    // allowance; claiming the default web-sized pool here can push past the
    // plan's per-role limit and abort the run before it starts. An explicit
    // DB_POOL_MAX_SIZE still wins.
    database::set_default_pool_max_size(2);

    println!("Running one-off database seed");
    let now = Instant::now();
    database::seed();
    println!("Seed finished in {}s.", now.elapsed().as_secs());
}
