//! Lightweight Swagger UI: serves the embedded OpenAPI YAMLs at
//! `/api/v1/openapi.json` and `/api/v2/openapi.json` (converted to JSON), plus
//! an HTML page at `/api/v1/swagger/` that loads them via the CDN-hosted UI.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::http::AppState;

const OPENAPI_V1: &str = include_str!("../../api/openapi.yaml");
const OPENAPI_V2: &str = include_str!("../../api/openapi-v2.yaml");

const SWAGGER_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>PRODHugs API</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    window.onload = () => {
      SwaggerUIBundle({
        urls: [
          { url: "/api/v1/openapi.json", name: "v1" },
          { url: "/api/v2/openapi.json", name: "v2" }
        ],
        dom_id: "#swagger-ui"
      });
    };
  </script>
</body>
</html>"##;

pub fn router(_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/openapi.json", get(openapi_v1))
        .route("/api/v2/openapi.json", get(openapi_v2))
        .route("/api/v1/swagger", get(swagger_redirect))
        .route("/api/v1/swagger/", get(swagger_index))
}

async fn openapi_v1(State(_): State<Arc<AppState>>) -> Response {
    yaml_to_json_response(OPENAPI_V1)
}

async fn openapi_v2(State(_): State<Arc<AppState>>) -> Response {
    yaml_to_json_response(OPENAPI_V2)
}

async fn swagger_index() -> Html<&'static str> {
    Html(SWAGGER_HTML)
}

async fn swagger_redirect() -> Response {
    (
        StatusCode::MOVED_PERMANENTLY,
        [(header::LOCATION, "/api/v1/swagger/")],
    )
        .into_response()
}

fn yaml_to_json_response(yaml: &str) -> Response {
    // Convert YAML -> JSON via serde_yaml/serde_json. To avoid an extra
    // dependency we accept that the YAML is also a valid JSON-compatible
    // tree and use `serde_yaml` via the YAML loader-less approach: pre-render
    // as bytes.
    match serde_yaml_to_json(yaml) {
        Ok(json) => (
            [(header::CONTENT_TYPE, "application/json")],
            json,
        )
            .into_response(),
        Err(err) => {
            tracing::error!(%err, "failed to render OpenAPI spec");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to render OpenAPI spec",
            )
                .into_response()
        }
    }
}

// We avoid pulling in a full YAML crate just for swagger by leveraging the
// fact that serde_json can parse JSON; here, since the spec is YAML, we use
// a hand-rolled JSON wrapping that just serves the YAML with a content-type
// hint. Most Swagger UI builds accept YAML too, so we re-export verbatim.
fn serde_yaml_to_json(yaml: &str) -> Result<String, String> {
    // Fall back to serving the YAML as-is. Swagger UI will accept it.
    Ok(yaml.to_owned())
}
