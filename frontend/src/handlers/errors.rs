
use actix_web::{web, get, HttpResponse, HttpRequest, Responder};
use actix_identity::Identity;
use crate::{AppData, generate_basic_context};


pub async fn f404(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (mut ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());

    let uri_path = req.uri().path();
    ctx.insert("path", &uri_path);

    let rendered = data.tmpl.render("errors/404.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/not_found")]
pub async fn not_found(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());

    let rendered = data.tmpl.render("errors/not_found.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/internal_server_error")]
pub async fn internal_server_error(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
     
    req: HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());

    let rendered = data.tmpl.render("errors/internal_server_error.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}

#[get("/{lang}/not_authorized")]
pub async fn not_authorized(
    web::Path(lang): web::Path<String>,
    data: web::Data<AppData>,
     
    req:HttpRequest,
    id: Identity,
) -> impl Responder {

    let (ctx, _, _, _) = generate_basic_context(id, &lang, req.uri().path());

    let rendered = data.tmpl.render("errors/not_authorized.html", &ctx).unwrap();
    HttpResponse::Ok().body(rendered)
}