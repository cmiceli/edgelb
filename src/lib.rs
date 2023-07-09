use serde_json::json;
use serde::{Deserialize, Serialize};
use rand::Rng;
use regex::Regex;
use url::Url;
use worker::*;

mod utils;
mod request;

#[derive(Serialize, Deserialize)]
enum Matcher {
    path(String),
    header(String),
}

#[derive(Serialize, Deserialize)]
struct HTTPRedirect {
    code: i32,
    location: String,
}

#[derive(Serialize, Deserialize)]
struct ServerPool {
    host: String,
    weight: u32,
}

#[derive(Serialize, Deserialize)]
struct Proxy {
    name: String,
    downstreams: list<ServerPool>,
}

#[derive(Serialize, Deserialize)]
enum Responder {
    http_code(HTTPRedirect),
    proxy(Proxy),
}

#[derive(Serialize, Deserialize)]
struct Route {
    matcher: Matcher,
    response: Responder,
}

#[derive(Serialize, Deserialize)]
struct Config {
    name: String,
    routes: Vec<Route>
}

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    // Optionally, get more helpful error messages written to the console in the case of a panic.
    utils::set_panic_hook();

    // Optionally, use the Router to handle matching endpoints, use ":name" placeholders, or "*name"
    // catch-alls to match on specific patterns. Alternatively, use `Router::with_data(D)` to
    // provide arbitrary data that will be accessible in each route via the `ctx.data()` method.

    let bytes = include_bytes!("config.json");
    let confStr = String::from_utf8_lossy(bytes);
    let cfg = serde_json::from_str::<Config>(&confStr).unwrap();
    // Add as many routes as your Worker needs! Each route will get a `Request` for handling HTTP
    // functionality and a `RouteContext` which you can use to  and get route parameters and
    // Environment bindings like KV Stores, Durable Objects, Secrets, and Variables.
    let mut found = false;
    for r in cfg.routes.into_iter() {
        console_log!("running route");
        match r.matcher {
            Matcher::path(p) => {
                let re = Regex::new(&p).unwrap();
                if re.is_match(&req.path()) {
                    found = true;
                }
            },
            Matcher::header(h) => {
                match req.headers().has(&h) {
                    Ok(v) => found = v,
                    Err(_) => return Response::error("Failed to get headers", 500),
                }
            },
        }
        if found {
            match r.response {
                Responder::http_code(code) => {
                    return Response::redirect(Url::parse(&code.location)?);
                },
                Responder::proxy(proxy) => {
                    //
                    let mut rng = rand::thread_rng();
                    let mut num = rng.gen_range(0..100);
                    let mut downstream = proxy.downstreams[0];
                    for i in proxy.downstreams.into_iter() {
                        if i.weight < num {
                            downstream = i;
                        }
                        num = num - i.weight;
                    }
                    return request::connect_send(downstream.host, req, env, ctx).await;
                },
            }
        }
    }
    return Response::error("Server Misconfigured", 500);
}

