// src/auth.rs — session-based authentication for admin routes.

use crate::configuration::AdminSettings;
use crate::template::TemplateRenderer;
use actix_identity::Identity;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::{HttpResponse, web::{self, Data}};
use serde::Serialize;

/// Check if the current request is authenticated. Returns Ok(()) if logged in,
/// or Err with a redirect to /login if not. Call this at the top of admin handlers.
pub fn require_login(identity: &Identity) -> Result<(), actix_web::Error> {
    match identity.id() {
        Ok(_) => Ok(()),
        Err(_) => Err(RedirectToLogin.into()),
    }
}

/// A custom error that redirects to /login. This lets us use `?` in handlers
/// that return `Result<_, actix_web::Error>`.
#[derive(Debug)]
struct RedirectToLogin;

impl std::fmt::Display for RedirectToLogin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "authentication required")
    }
}

impl actix_web::ResponseError for RedirectToLogin {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::SeeOther()
            .insert_header(("Location", "/login"))
            .finish()
    }
}

/// POST /login — validate credentials, attach identity.
pub async fn post_login(
    req: HttpRequest,
    params: web::Form<LoginForm>,
    admin: Data<AdminSettings>,
) -> HttpResponse {
    if params.username == admin.username && params.password == admin.password {
        match Identity::login(&req.extensions(), "admin".to_string()) {
            Ok(_) => HttpResponse::SeeOther()
                .insert_header(("Location", "/gallery"))
                .finish(),
            Err(_) => HttpResponse::SeeOther()
                .insert_header(("Location", "/login?error=1"))
                .finish(),
        }
    } else {
        HttpResponse::SeeOther()
            .insert_header(("Location", "/login?error=1"))
            .finish()
    }
}

/// GET /login — render the styled login page.
pub async fn get_login_page(
    templates: Data<Box<dyn TemplateRenderer>>,
    query: web::Query<LoginQuery>,
) -> Result<HttpResponse, actix_web::Error> {
    #[derive(Serialize)]
    struct Context {
        title: &'static str,
        header: &'static str,
        sub_header: &'static str,
        error: bool,
    }

    let context = Context {
        title: "Login",
        header: "Metallian Photos",
        sub_header: "Admin Access",
        error: query.error.is_some(),
    };

    let json_context = serde_json::to_value(&context)?;
    let html = templates.render("login.html", &json_context)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}

/// GET /logout — clear identity and redirect to galleries.
pub async fn get_logout(identity: Identity) -> HttpResponse {
    identity.logout();
    HttpResponse::SeeOther()
        .insert_header(("Location", "/galleries"))
        .finish()
}

#[derive(serde::Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginQuery {
    pub error: Option<String>,
}
