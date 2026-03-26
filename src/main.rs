use reqwest;
use scraper::{Html, Selector};
use serde::Serialize;
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;

#[derive(Serialize)]
pub struct LinkPreviewData {
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
}

const API_KEY: &str = "nsec14x8a0eadlatvegphjh5deq9aa7ajzvldpukd5msvjkuaakuf7wnqf6nm6h";

#[get("/preview")]
async fn preview(req: actix_web::HttpRequest, query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let key = req.headers().get("X-API-Key").and_then(|v| v.to_str().ok());
    if key != Some(API_KEY) {
        return HttpResponse::Unauthorized().body("Invalid API key");
    }

    let url = match query.get("url") {
        Some(u) => u,
        None => return HttpResponse::BadRequest().body("Missing url parameter"),
    };

    let html = match fetch_html(url) {
        Ok(h) => h,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch HTML"),
    };

    match parse_link_preview(&html) {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(_) => HttpResponse::InternalServerError().body("Failed to parse preview"),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .service(preview)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn parse_link_preview(html: &str) -> Result<LinkPreviewData, String> {
    let document = Html::parse_document(html);

    // Title
    let title_selector = Selector::parse("title").map_err(|e| e.to_string())?;
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join(" ").trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "".to_string());

    // Description (meta[name="description"])
    let desc_selector = Selector::parse("meta[name='description']").map_err(|e| e.to_string())?;
    let description = document
        .select(&desc_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string());

    // Image (meta[property="og:image"])
    let img_selector = Selector::parse("meta[property='og:image']").map_err(|e| e.to_string())?;
    let image_url = document
        .select(&img_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string());

    Ok(LinkPreviewData {
        title,
        description,
        image_url,
    })
}

fn fetch_html(url: &str) -> Result<String, reqwest::Error> {
    let response = reqwest::blocking::get(url)?;
    let body = response.text()?;
    Ok(body)
}

