use log::{debug, error, info};

use tiny_http::{Method, Request, Response, ResponseBox};
use velo::wahoo::{self, WahooWebhook};
use velo::AppConfig;

pub fn wahoo_webhook(
    config: &AppConfig,
    request: &mut Request,
) -> Result<ResponseBox, anyhow::Error> {
    let webhook: WahooWebhook = serde_json::from_reader(request.as_reader())?;
    let expected_webhook_token = &config.wahoo_webhook_token[..];

    if webhook.webhook_token != expected_webhook_token {
        error!(
            "Unexpected webhook token: {}, expected {}",
            webhook.webhook_token, expected_webhook_token
        );

        let response = Response::from_string("Invalid webhook token")
            .with_status_code(400)
            .boxed();
        return Ok(response);
    }

    debug!("Webhook Data: {webhook:?}");
    wahoo::handle_webhook(&config.sqlite_directory, &webhook)
        .map(|_| Response::new_empty(200.into()).boxed())
}

pub fn main() -> Result<(), anyhow::Error> {
    use tiny_http::Server;
    env_logger::init();

    let config = AppConfig::from_env().expect("Failed to load AppConfig from env");
    if config.sqlite_directory.exists() == false {
        std::fs::create_dir_all(&config.sqlite_directory)?;
    }

    let server = Server::http("0.0.0.0:8000").unwrap();

    for mut request in server.incoming_requests() {
        info!(
            "received request! method: {:?}, url: {:?}, headers: {:?}",
            request.method(),
            request.url(),
            request.headers()
        );

        let response = match (request.method(), request.url()) {
            (Method::Post, "/v1/wahoo/webhook") => match wahoo_webhook(&config, &mut request) {
                Ok(response) => response,
                Err(e) => Response::from_string(e.to_string())
                    .with_status_code(400)
                    .boxed(),
            },
            _ => Response::from_string("Invalid Request")
                .with_status_code(400)
                .boxed(),
        };

        request.respond(response)?;
    }

    Ok(())
}
