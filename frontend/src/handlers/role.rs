use actix_session::UserSession;
use serde::{Deserialize, Serialize};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web, ResponseError};
use actix_identity::{Identity};
use uuid::Uuid;


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_role_by_id};

#[derive(Deserialize, Debug)]
pub struct AddRoleForm {
    pub team_id: Uuid,
    pub title_en: String,
    pub title_fr: String,
    pub active: bool,
    pub hr_roup: String,
    pub hr_level: i32,
    pub requirements: Vec<(String, String)>,
}

#[get("/{lang}/role/{role_id}")]
pub async fn role_by_id(
    data: web::Data<AppData>,
    id: Identity,
    web::Path((lang, role_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_role_by_id(role_id, bearer, &data.api_url)
        .expect("Unable to get people");

    ctx.insert("role", &r.role_by_id);

    let rendered = data.tmpl.render("role/role.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/create_role")]
pub async fn create_role(
    data: web::Data<AppData>,
    id: Identity,
    web::Path((lang, role_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let rendered = data.tmpl.render("role/role.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[post("/{lang}/role_submit")]
pub async fn role_submit(
    data: web::Data<AppData>,
    id: Identity,
    form: web::Form<AddRoleForm>,
    web::Path((lang, role_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let rendered = data.tmpl.render("role/role.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}