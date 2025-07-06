use actix_session::UserSession;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, ResponseError};
use actix_identity::{Identity};


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_work_by_id};

#[get("/{lang}/work/{work_id}")]
pub async fn work_by_id(
    data: web::Data<AppData>,
    id: Identity,
    web::Path((lang, work_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_work_by_id(work_id, bearer, &data.api_url)
        .expect("Unable to get people");

    ctx.insert("work", &r.work_by_id);

    let rendered = data.tmpl.render("work/work.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}