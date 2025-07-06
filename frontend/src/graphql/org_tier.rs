use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest;

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tier_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTierById;

pub fn get_org_tier_by_id(id: UUID, bearer: String, api_url: &str) -> Result<org_tier_by_id::ResponseData, Box<dyn Error>> {

    let request_body = OrgTierById::build_query(org_tier_by_id::Variables {
        id,
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()?;

    let response_body: Response<org_tier_by_id::ResponseData> = res.json()?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data
        .expect("missing response data");

    // serve HTML page with response_body
    Ok(response)
}

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/org_tiers/org_tiers_by_org_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct OrgTiersByOrgId;
pub fn get_org_tiers_by_org_id(id: UUID, bearer: String, api_url: &str) -> Result<org_tiers_by_org_id::ResponseData, Box<dyn Error>> {

    let request_body = OrgTiersByOrgId::build_query(org_tiers_by_org_id::Variables {
        id,
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()?;

    let response_body: Response<org_tiers_by_org_id::ResponseData> = res.json()?;

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    };

    let response = response_body.data
        .expect("missing response data");

    // serve HTML page with response_body
    Ok(response)
}