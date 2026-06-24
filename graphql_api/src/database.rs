use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use lazy_static::lazy_static;
use r2d2::{self};
use std::env;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::database_utils::pre_populate_db_schema;
use crate::database_utils::pre_populate_skills;
use errors::CustomError;

pub type PostgresPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

use crate::models::{User, UserData, InsertableUser, Organization};


const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

lazy_static! {
    pub static ref POOL: PostgresPool = {
        let db_url = env::var("DATABASE_URL").expect("Database url not set");
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        PostgresPool::new(manager).expect("Failed to create DB Pool")
    };
}

fn run_migration(conn: &mut PgConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

/// Web-boot path: run migrations and make sure the admin user exists, then
/// return immediately so the HTTP server can bind to $PORT well within
/// Heroku's 60s boot window. Heavy demo-data generation is intentionally NOT
/// run here — it lives in `seed()` and is invoked as a one-off command (see
/// `main.rs` / the `--seed` flag) so it can't trigger an R10 boot timeout.
pub fn init() {

    lazy_static::initialize(&POOL);
    let mut conn = connection().expect("Failed to get DB connection");
    run_migration(&mut conn);

    ensure_admin();
}

/// Create the admin user from the ADMIN_* env vars if it doesn't already
/// exist. Cheap and idempotent; safe to call on every boot.
fn ensure_admin() {
    let admin_name = env::var("ADMIN_NAME").expect("Unable to load admin name");
    let admin_email = env::var("ADMIN_EMAIL").expect("Unable to load admin email");
    let admin_pwd = env::var("ADMIN_PASSWORD").expect("Unable to load admin password");

    match User::get_by_email(&admin_email) {
        Ok(u) => println!("Admin exists {:?} - bypass admin setup", &u),
        Err(_e) => {
            let admin_data = UserData {
                name: admin_name.trim().to_owned(),
                email: admin_email.trim().to_owned(),
                password: admin_pwd.trim().to_owned(),
                role: "ADMIN".to_owned(),
                account_type: None,
            };

            let test_admin = InsertableUser::from(admin_data);

            let admin = User::create(test_admin)
                .expect("Unable to create admin");

            println!("Admin created: {:?}", &admin);
        }
    }
}

/// Heavy, one-off demo-data seeding (skills, org structure, people,
/// capabilities, validations, publications, tasks, products). This can take
/// well over a minute against a remote database, so it must NOT run during web
/// dyno boot. Invoke it as a one-off process instead, e.g. on Heroku:
///
///     heroku run "./target/release/graphql_api --seed" -a <api-app>
///
/// Guarded so it is a no-op if the database already contains seed data, making
/// it safe to re-run. To re-seed from scratch, reset the database first.
pub fn seed() {
    lazy_static::initialize(&POOL);
    let mut conn = connection().expect("Failed to get DB connection");
    run_migration(&mut conn);

    // Make sure the admin exists; the seed references existing users as
    // validation authorities.
    ensure_admin();

    // Idempotency guard: if organizations already exist, assume the database
    // has been seeded and bail out rather than duplicating data.
    match Organization::get_all() {
        Ok(orgs) if !orgs.is_empty() => {
            println!(
                "Database already contains {} organization(s) - skipping seed. \
                 Reset the database first if you want to re-seed.",
                orgs.len()
            );
            return;
        }
        _ => {}
    }

    let _res = pre_populate_skills().expect("error in populating skills");

    println!("Pre-populating database");
    let _res = pre_populate_db_schema()
        .expect("Unable to pre-populate database");

    println!("Seeding complete.");
}

pub fn connection() -> Result<DbConnection, CustomError> {
    POOL.get()
        .map_err(|e| CustomError::new(500, format!("Failed getting db connection: {}", e)))
}