use actix_session::UserSession;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, ResponseError};
use actix_identity::{Identity};


use crate::{AppData, generate_basic_context};
use crate::graphql::{get_task_by_id};

#[get("/{lang}/task/{task_id}")]
pub async fn task_by_id(
    data: web::Data<AppData>,
    id: Identity,
    web::Path((lang, task_id)): web::Path<(String, String)>,
    
    req:HttpRequest) -> impl Responder {

    let (mut ctx, user, lang, path) = generate_basic_context(id, &lang, req.uri().path());

    let bearer = match req.get_session().get::<String>("bearer").unwrap() {
        Some(s) => s,
        None => "".to_string(),
    };

    let r = get_task_by_id(task_id, bearer, &data.api_url)
        .expect("Unable to get people");

    ctx.insert("task", &r.task_by_id);

    let rendered = data.tmpl.render("task/task.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}