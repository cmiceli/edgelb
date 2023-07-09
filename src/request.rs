use std::str::Utf8Error;

use worker::*;

async fn make_request(
    mut sender: hyper::client::conn::SendRequest<hyper::Body>,
    request: hyper::Request<hyper::Body>,
) -> Result<Response> {
    // Send and recieve HTTP request
    let hyper_response = sender
        .send_request(request)
        .await
        .map_err(map_hyper_error)?;

    // Convert back to worker::Response
    let buf = hyper::body::to_bytes(hyper_response)
        .await
        .map_err(map_hyper_error)?;
    let text = std::str::from_utf8(&buf).map_err(map_utf8_error)?;
    let mut response = Response::ok(text)?;
    response.headers_mut().append("Content-Type", "text/html")?;
    Ok(response)
}

async fn connect_send(hostname: String, req: Request, _env: Env, _ctx: Context) -> worker::Result<Response> {
    set_panic_hook();

    let socket = Socket::builder()
        .secure_transport(SecureTransport::On)
        .connect(&hostname, 443)?;

    let (sender, connection) = hyper::client::conn::handshake(socket)
        .await
        .map_err(map_hyper_error)?;

    req.headers_mut()?.set("Host", &hostname);

    tokio::select!(
        res = connection => {
            console_error!("Connection exited: {:?}", res);
            Err(worker::Error::RustError("Connection exited".to_string()))
        },
        result = make_request(sender, req) => result
    )
}

fn map_utf8_error(error: Utf8Error) -> worker::Error {
    worker::Error::RustError(format!("Utf8Error: {:?}", error))
}

fn map_hyper_error(error: hyper::Error) -> worker::Error {
    worker::Error::RustError(format!("hyper::Error: {:?}", error))
}

