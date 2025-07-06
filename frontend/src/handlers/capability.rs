use actix_session::UserSession;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, ResponseError};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};
use crate::graphql::get_capability_by_name_and_level;

#[get("/{lang}/capability_search/{name}/{level}")]
pub async fn capability_search(
    web::Path((lang, name, level)): web::Path<(String, String, String)>,
    data: web::Data<AppData>,
    req: HttpRequest, 
    id: Identity,
) -> impl Responder {

    println!("CALL CAPABILITY SEARCH");

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };
    
    // query graphql API
    let results = get_capability_by_name_and_level(
        name.to_lowercase().trim().to_string(),
        level.clone(),
        bearer.clone(),
        &data.api_url,
    )
    .expect("Unable to find capabilities");

    println!("{:?}", &results);
             
    ctx.insert("capabilities", &results.capabilities_by_name_and_level);
    ctx.insert("name", &name.to_owned());
    ctx.insert("level", &level);

    let rendered = data.tmpl.render("capability/capability_search_results.html", &ctx).unwrap();
    HttpResponse::Ok()
        .header("Bearer", bearer)
        .body(rendered)
}