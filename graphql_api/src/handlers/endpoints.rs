use actix_web::{web, HttpResponse, HttpRequest, Result};
use async_graphql::http::{GraphiQLSource};
use async_graphql::Schema;

use async_graphql_actix_web::{GraphQLSubscription,
    GraphQLRequest, GraphQLResponse};

use async_graphql::dataloader::DataLoader;

use crate::models;
use crate::graphql::{AppSchema};
use crate::graphql::loaders::{
    PersonLoader, TeamLoader, RoleLoader, TaskLoader, ProductLoader,
    WorkByRoleLoader, RequirementsByRoleLoader, EffortByRoleLoader, AssignmentsByRoleLoader,
    PayRatesLoader,
};


pub async fn playground_handler() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub async fn graphql(
    schema: web::Data<AppSchema>,
    http_request: HttpRequest,
    req: GraphQLRequest,
) -> GraphQLResponse {
    
    let mut query = req.into_inner();

    let maybe_role_id = models::get_claim(http_request);

    // insert claim data into query or error for response
    match maybe_role_id {
        Ok((role, uuid, exp_time)) => {
            query = query.data(role);
            query = query.data(uuid);
            query = query.data(exp_time)
        },
        Err(e) => {
            query = query.data(e);
        }
    };

    // Per-request DataLoaders. Scoped to this request so their batching/cache
    // never leaks rows across requests (which would go stale after mutations).
    query = query
        .data(DataLoader::new(PersonLoader, actix_web::rt::spawn))
        .data(DataLoader::new(TeamLoader, actix_web::rt::spawn))
        .data(DataLoader::new(RoleLoader, actix_web::rt::spawn))
        .data(DataLoader::new(TaskLoader, actix_web::rt::spawn))
        .data(DataLoader::new(ProductLoader, actix_web::rt::spawn))
        .data(DataLoader::new(WorkByRoleLoader, actix_web::rt::spawn))
        .data(DataLoader::new(RequirementsByRoleLoader, actix_web::rt::spawn))
        .data(DataLoader::new(EffortByRoleLoader, actix_web::rt::spawn))
        .data(DataLoader::new(AssignmentsByRoleLoader, actix_web::rt::spawn))
        .data(DataLoader::new(PayRatesLoader, actix_web::rt::spawn));

    schema.execute(query).await.into()
}

pub async fn graphql_ws(
    schema: web::Data<AppSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
}
