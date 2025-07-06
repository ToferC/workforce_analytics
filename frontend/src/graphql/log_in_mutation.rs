use graphql_client::{GraphQLQuery, Response};
use serde::{Serialize, Deserialize};
use std::error::Error;
use reqwest;

use crate::graphql::log_in_mutation;
use crate::graphql::log_in::LoginQuery;

type UUID = String;
#[derive(GraphQLQuery, Serialize, Deserialize)]
#[graphql(
    schema_path = "schema.graphql",
    query_path = "queries/log_in.graphql",
    response_derives = "Debug, Serialize, PartialEq"
)]
pub struct LogIn;


pub fn login(email: String, password: String, api_url: &str) -> Result<log_in::ResponseData, Box<dyn Error>> {

    let auth_data = log_in_mutation::LoginQuery{
        email, 
        password,
    };

    let request_body = LogIn::build_query(log_in::Variables {
        auth_data: auth_data,
    });

    let client = reqwest::blocking::Client::new();
    let mut res = client.post(api_url).json(&request_body).send()?;

    let response_body: Response<log_in::ResponseData> = res.json()?;

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