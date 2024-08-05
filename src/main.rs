use actix_web::{get, App, HttpRequest, HttpServer, HttpResponse, Result};
use actix_web::web::Bytes;
use reqwest::Client;
use std::collections::HashMap;
use url::Url;

#[get("/proxy")]
async fn proxy(req: HttpRequest) -> Result<HttpResponse> {
    // Extract query string and parse it into a HashMap
    let query: HashMap<String, String> = req.query_string()
        .split('&')
        .filter_map(|pair| {
            let mut split = pair.split('=');
            if let (Some(key), Some(value)) = (split.next(), split.next()) {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect();

    // Check if the 'destination' parameter is provided
    if let Some(destination) = query.get("destination") {
        // Decode the URI-encoded destination URL
        let decoded_url = match urlencoding::decode(destination) {
            Ok(url) => url.into_owned(),
            Err(_) => return Ok(HttpResponse::BadRequest().body("Invalid destination URL")),
        };

        // Parse the decoded URL to validate it
        if let Ok(url) = Url::parse(&decoded_url) {
            // Make the request to the target URL
            let client = Client::new();
            let mut response = match client.get(url).send().await {
                Ok(resp) => resp,
                Err(_) => return Ok(HttpResponse::InternalServerError().body("Failed to fetch the destination URL")),
            };

            let mut client_response = HttpResponse::Ok();

            // Copy headers from the upstream response to the client response
            for (key, value) in response.headers().iter() {
                client_response.append_header((key.as_str(), value.to_str().unwrap_or("")));
            }

            // Create a streaming response
            let response_stream = async_stream::stream! {
                while let Some(chunk) = response.chunk().await.unwrap_or(None) {
                    yield Ok::<Bytes, actix_web::Error>(Bytes::from(chunk));
                }
            };

            return Ok(client_response.streaming(response_stream));
        } else {
            return Ok(HttpResponse::BadRequest().body("Invalid destination URL"));
        }
    } else {
        Ok(HttpResponse::BadRequest().body("Missing destination query parameter"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(proxy)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
