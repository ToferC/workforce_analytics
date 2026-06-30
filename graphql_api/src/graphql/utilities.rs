use std::sync::{Arc, Mutex};
use crate::database::PostgresPool;

use async_graphql::*;
use diesel::{PgConnection};
use diesel::r2d2::ConnectionManager;
use r2d2::PooledConnection;

use crate::graphql::{Mutation, query::Query}; // Removed Subscription

// use crate::kafka::{create_producer};

pub fn graphql_translate<T>(res: Result<T, diesel::result::Error>) -> FieldResult<T> {
    match res {
        Ok(t) => Ok(t),
        Err(e) => Err(FieldError::from(e)),
    }
}

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema_with_context(pg_pool: PostgresPool) -> async_graphql::Schema<Query, Mutation, EmptySubscription> {
    
    //let cloned_conn = pg_pool.clone().get().expect("Unable to connect to db");
    
    let arc_pool = Arc::new(pg_pool);

    /*
    let countries = Arc::new(Mutex::new(Country::load_into_hash(&cloned_conn)));
    let places = Arc::new(Mutex::new(Place::load_into_hash(&cloned_conn)));
    let vaccines = Vaccine::load_into_hash(&cloned_conn);
    */
    let identity: Option<String> = None;

    let kafka_consumer_counter = Mutex::new(0);

    // Guard against pathological queries (deep recursive nesting through
    // role -> manager -> team -> owner cycles, or huge field fan-out) that
    // would amplify the resolvers' per-field database work. The deepest
    // legitimate client query nests ~6 levels, so 15 leaves generous
    // headroom. Both limits are env-overridable so a genuinely larger query
    // can be unblocked without a redeploy.
    let max_depth = std::env::var("GRAPHQL_MAX_DEPTH")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(15);
    let max_complexity = std::env::var("GRAPHQL_MAX_COMPLEXITY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1000);

    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .limit_depth(max_depth)
        .limit_complexity(max_complexity)
        // Database connection
        .data(arc_pool)
        // Live cached data -> may want to remove once dataloaders in place
        /*
        .data(countries)
        .data(places)
        .data(vaccines)
         */
        .data(identity)
        // Kafka
        // .data(create_producer())
        .data(kafka_consumer_counter)
        .finish()
}

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn get_connection_from_context(ctx: &Context<'_>) -> Conn {
    ctx.data::<Arc<PostgresPool>>()
        .expect("Can't get pool")
        .get()
        .expect("Can't get DB connection")
}