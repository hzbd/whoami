use std::env;
use std::net::SocketAddr;
use warp::Filter;
use gethostname::gethostname;
use serde::Serialize;

#[derive(Serialize)]
struct WhoamiResponse {
    hostname: String,
}

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16 number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let text_route = warp::get()
        .and(warp::path::end())
        .map(|| {
            let hostname = gethostname().into_string().unwrap_or_else(|_| "unknown_hostname".to_string());
            println!("I'm {} (request for text)", &hostname);
            format!("I'm {}", hostname)
        });

    let json_route = warp::get()
        .and(warp::path("json"))
        .and(warp::path::end())
        .map(|| {
            let hostname = gethostname().into_string().unwrap_or_else(|_| "unknown_hostname".to_string());
            println!("I'm {} (request for json)", &hostname);
            let response = WhoamiResponse { hostname };
            warp::reply::json(&response)
        });

    let routes = text_route.or(json_route);

    println!("Listening on http://{}", addr);
    println!("Endpoints: GET / and GET /json");

    warp::serve(routes).run(addr).await;
}
