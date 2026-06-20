//! Print the GraphQL SDL to stdout, so `schema.graphqls` can be regenerated
//! without a running database:
//!
//!   cargo run --example dump_schema -p graphql_api > schema.graphqls
//!
//! The schema is built purely from the type/resolver definitions; the database
//! pool is only stored as context data, so an unchecked (non-connecting) pool is
//! enough.

use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use graphql_api::graphql::create_schema_with_context;

fn main() {
    let manager = ConnectionManager::<PgConnection>::new("postgres://unused/unused");
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);

    let schema = create_schema_with_context(pool);
    print!("{}", schema.sdl());
}
