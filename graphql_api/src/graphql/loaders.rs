//! Per-request `DataLoader`s that batch the single-row lookups the GraphQL
//! resolvers would otherwise issue one at a time (the classic N+1 pattern).
//!
//! Each loader collapses many `load_one` calls made while resolving a list
//! into a single `WHERE id = ANY($1)` query. They are registered per request
//! in the `/graphql` handler (see `handlers::endpoints::graphql`) so their
//! cache never outlives a single request — important for correctness, since a
//! schema-global loader would serve stale rows after a mutation.

use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use uuid::Uuid;

use crate::models::{Person, Product, Role, Task, Team, Work};

/// `Loader::Error` must be `Send + Clone + 'static`. `async_graphql::Error`
/// already satisfies that, and is what the model getters return, so it passes
/// straight through.
type LoadError = async_graphql::Error;

/// Defines a loader keyed by primary id over a batched `get_by_ids` getter.
macro_rules! id_loader {
    ($(#[$m:meta])* $name:ident => $value:ty, $getter:path) => {
        $(#[$m])*
        pub struct $name;

        impl Loader<Uuid> for $name {
            type Value = $value;
            type Error = LoadError;

            async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, $value>, Self::Error> {
                let rows = $getter(&keys.to_vec())?;
                Ok(rows.into_iter().map(|row| (row.id, row)).collect())
            }
        }
    };
}

id_loader!(
    /// Batches `Person::get_by_id` (e.g. `Role::person`).
    PersonLoader => Person, Person::get_by_ids
);
id_loader!(
    /// Batches `Team::get_by_id` (e.g. `Role::team`).
    TeamLoader => Team, Team::get_by_ids
);
id_loader!(
    /// Batches `Role::get_by_id` (e.g. `Work::role`, `Team::owner`).
    RoleLoader => Role, Role::get_by_ids
);
id_loader!(
    /// Batches `Task::get_by_id` (e.g. `Work::task`).
    TaskLoader => Task, Task::get_by_ids
);
id_loader!(
    /// Batches `Product::get_by_id` (e.g. `Task::product`).
    ProductLoader => Product, Product::get_by_ids
);

/// One-to-many loader: batches `Work::get_by_role_id` (e.g. `Role::work`),
/// grouping the flat result back into a list per role.
pub struct WorkByRoleLoader;

impl Loader<Uuid> for WorkByRoleLoader {
    type Value = Vec<Work>;
    type Error = LoadError;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Vec<Work>>, Self::Error> {
        let rows = Work::get_by_role_ids(keys)?;
        let mut grouped: HashMap<Uuid, Vec<Work>> = HashMap::new();
        for work in rows {
            if let Some(role_id) = work.role_id {
                grouped.entry(role_id).or_default().push(work);
            }
        }
        Ok(grouped)
    }
}
