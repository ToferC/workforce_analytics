use actix_session::UserSession;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, ResponseError};
use actix_identity::{Identity};


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_org_tier_by_id};

#[get("/{lang}/org_tier/{org_tier_id}")]
pub async fn org_tier_by_id(
    data: web::Data<AppData>,
    id: Identity,
    web::Path((lang, org_tier_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_org_tier_by_id(org_tier_id, bearer, &data.api_url)
        .expect("Unable to get people");

    ctx.insert("org_tier", &r.org_tier_by_id);

    let rendered = data.tmpl.render("org_tier/org_tier.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}