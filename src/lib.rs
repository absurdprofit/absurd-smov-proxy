use std::collections::HashMap;

use spin_sdk::http::{
    IncomingRequest, IncomingResponse, Method, OutgoingResponse, Request, ResponseOutparam, SendError, StatusCode
};
use spin_sdk::http_component;
use url::Url;
use futures_util::sink::SinkExt;
use futures_util::StreamExt;

pub fn extract_query_params(uri: &str) -> HashMap<String, String> {
    if let Ok(url) = Url::parse(uri) {
        // Step 3: Extract query parameters into a HashMap
        return url.query_pairs().into_owned().collect();
    }

    // Return an empty HashMap if no query string is present or an error occurred
    HashMap::new()
}

pub async fn create_error_response(status: StatusCode, message: &str) -> OutgoingResponse {
    let headers = spin_sdk::http::Headers::new();
    let _ = headers.set(&String::from("content-type"), &["text/plain".as_bytes().to_vec()]);
    let response = OutgoingResponse::new(headers);
    let _ = response.set_status_code(status);
    let mut body = response.take_body();
    let _ = body.send(message.as_bytes().to_vec()).await;
    return response;
}

/// A simple Spin HTTP component.
#[http_component]
async fn handle(req: IncomingRequest, response_out: ResponseOutparam) {
    // Check if the 'destination' parameter is provided
    if let Some(destination) = extract_query_params(&req.uri()).get("destination") {
        // Decode the URI-encoded destination URL
        let decoded_url = match urlencoding::decode(destination) {
            Ok(url) => url.into_owned(),
            Err(_) => {
                let response = create_error_response(400, "Invalid destination URL").await;
                return response_out.set(response);
            }
        };

        // Parse the decoded URL to validate it
        if let Ok(url) = Url::parse(&decoded_url) {
            // Make the request to the target URL
            let request = Request::builder()
                .method(Method::Get)
                .headers(req.headers())
                .uri(url)
                .build();

            let response: Result<IncomingResponse, SendError> = spin_sdk::http::send(request).await;
            let response = match response {
                Ok(resp) => resp,
                Err(_) => {
                    let response = create_error_response(500, "Failed to fetch the destination URL").await;
                    return response_out.set(response);
                }
            };

            let client_response = OutgoingResponse::new(
                response.headers(),
            );
            let _ = client_response.set_status_code(response.status());

            // Create a streaming response
            let mut body = client_response.take_body();
            let mut stream = response.take_body_stream();

            // Connect the OutgoingResponse to the ResponseOutparam.
            response_out.set(client_response);

            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        if let Err(e) = body.send(chunk).await {
                            eprintln!("Error sending chunk: {e:#}");
                        }
                    },
                    Err(e) => {
                        eprintln!("Error reading chunk: {e:#}");
                    }
                }
            }
        } else {
            let response = create_error_response(400, "Invalid destination URL").await;
            return response_out.set(response);
        }
    } else {
        let response = create_error_response(400, "Missing destination query parameter").await;
        return response_out.set(response);
    }
}
