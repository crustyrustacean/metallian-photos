// src/routes/frontend.rs

pub mod delete;
pub mod edit;
pub mod galleries;
pub mod gallery;
pub mod index;

/// GET * — catch-all 404 handler.
pub async fn not_found(
    templates: actix_web::web::Data<Box<dyn crate::template::TemplateRenderer>>,
    identity: Option<actix_identity::Identity>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    use serde::Serialize;
    let logged_in = identity.map(|i| i.id().is_ok()).unwrap_or(false);
    #[derive(Serialize)]
    struct Ctx {
        title: &'static str,
        header: &'static str,
        sub_header: &'static str,
        logged_in: bool,
        status_code: &'static str,
        message: &'static str,
    }
    let ctx = Ctx {
        title: "Not Found",
        header: "Metallian Photos",
        sub_header: "Concert Photo Archive",
        logged_in,
        status_code: "404",
        message: "The page you are looking for does not exist.",
    };
    let json_ctx = serde_json::to_value(&ctx)?;
    let html = templates.render("error.html", &json_ctx)?;
    Ok(actix_web::HttpResponse::NotFound().content_type("text/html").body(html))
}
pub mod upload;

pub use delete::*;
pub use edit::*;
pub use galleries::*;
pub use gallery::*;
pub use index::*;
pub use upload::*;