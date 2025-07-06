use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest;

type UUID = String;

#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/teams/team_by_id.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct TeamById;

pub fn get_team_by_id(id: UUID, bearer: String, api_url: &str) -> Result<team_by_id::ResponseData, Box<dyn Error>> {

    let request_body = TeamById::build_query(team_by_id::Variables {
        id,
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()?;

    let response_body: Response<team_by_id::ResponseData> = res.json()?;

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
    query_path = "queries/teams/all_teams.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct AllTeams;

pub fn all_teams(bearer: String, api_url: &str) -> Result<all_teams::ResponseData, Box<dyn Error>> {

    let request_body = AllTeams::build_query(all_teams::Variables {
    });

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(api_url)
        .header("Bearer", bearer)
        .json(&request_body)
        .send()?;

    let response_body: Response<all_teams::ResponseData> = res.json()?;

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