// src/main.rs

use axum::{
    Router,
    body::{self, Body},
    extract::{ConnectInfo, FromRequest, Multipart, Request},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::any,
};
use chrono::{DateTime, Utc};
use gethostname::gethostname;
use serde::Serialize;
use serde_json::{Value, json};
use std::{net::SocketAddr, time::Instant};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[derive(Serialize)]
struct FinalResponse {
    request_id: String,
    server: String,
    hostname: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    at: DateTime<Utc>,
    processing_time_ms: f64,
    ip_details: IpDetails,
    method: String,
    protocol: String,
    uri: String,
    body: Value,
}
#[derive(Serialize)]
struct IpDetails {
    client_ip: String,
    peer_ip: String,
    port: u16,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    let app = Router::new().fallback(any(inspector_handler)).layer(
        tower::ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(CompressionLayer::new()),
    );

    let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    info!("Starting HTTP server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to port 80");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn inspector_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<Body>,
) -> impl IntoResponse {
    let start = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let (parts, body) = req.into_parts();

    let hostname = gethostname().to_string_lossy().into_owned();
    let get_header = |key: &str| -> String {
        parts
            .headers
            .get(key)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string()
    };

    let xff_header = get_header("x-forwarded-for");
    let client_ip = if !xff_header.is_empty() {
        xff_header
            .split(',')
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    } else {
        get_header("x-real-ip")
    };
    let client_ip = if client_ip.is_empty() {
        addr.ip().to_string()
    } else {
        client_ip
    };

    let content_type_str = get_header("content-type");
    let body_value = if content_type_str.starts_with("multipart/form-data") {
        let req_for_multipart = Request::from_parts(parts.clone(), body);
        parse_multipart_body(req_for_multipart).await
    } else {
        parse_generic_body(&content_type_str, body).await
    };

    let server_string = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let response = FinalResponse {
        request_id,
        server: server_string,
        hostname,
        at: Utc::now(),
        processing_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        ip_details: IpDetails {
            client_ip,
            peer_ip: addr.ip().to_string(),
            port: addr.port(),
        },
        method: parts.method.to_string(),
        protocol: format!("{:?}", parts.version),
        uri: parts.uri.to_string(),
        body: body_value,
    };

    (StatusCode::OK, Json(response))
}

async fn parse_generic_body(content_type_str: &str, body: Body) -> Value {
    let mime_type: mime::Mime = content_type_str
        .parse()
        .unwrap_or(mime::APPLICATION_OCTET_STREAM);
    let bytes = match body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return json!({"error": "Failed to read body"}),
    };
    if bytes.is_empty() {
        return Value::Null;
    }
    match (mime_type.type_(), mime_type.subtype()) {
        (mime::APPLICATION, mime::JSON) => match serde_json::from_slice::<Value>(&bytes) {
            Ok(parsed_json) => json!({"format": "json", "content": parsed_json}),
            Err(e) => {
                json!({"format": "json", "error": "Failed to parse JSON", "details": e.to_string(), "raw_content": String::from_utf8_lossy(&bytes)})
            }
        },
        (mime::APPLICATION, mime::WWW_FORM_URLENCODED) => {
            match serde_urlencoded::from_bytes::<Value>(&bytes) {
                Ok(parsed_json) => parsed_json,
                Err(e) => {
                    json!({"error": "Failed to parse form-urlencoded", "details": e.to_string(), "raw_content": String::from_utf8_lossy(&bytes)})
                }
            }
        }
        (mime::APPLICATION, mime::XML) | (mime::TEXT, mime::XML) => {
            json!({"format": "xml", "content": String::from_utf8_lossy(&bytes)})
        }
        (mime::TEXT, _) => json!(String::from_utf8_lossy(&bytes)),
        _ => {
            json!({"format": "binary", "content_type": mime_type.to_string(), "size_bytes": bytes.len(), "content": "[binary data not displayed]"})
        }
    }
}
async fn parse_multipart_body(req: Request<Body>) -> Value {
    let mut multipart = match Multipart::from_request(req, &()).await {
        Ok(mp) => mp,
        Err(e) => return json!({"error": format!("Invalid multipart request: {}", e)}),
    };
    let mut text_fields = std::collections::HashMap::new();
    let mut files_ignored = vec![];
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("unnamed_field").to_string();
        if let Some(file_name) = field.file_name() {
            files_ignored
                .push(json!({ "field_name": name, "original_filename": file_name.to_string() }));
        } else if let Ok(text) = field.text().await {
            text_fields.insert(name, text);
        }
    }
    json!({ "text_fields": text_fields, "files_ignored": files_ignored })
}
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {}, }
    info!("Signal received, starting graceful shutdown...");
}
