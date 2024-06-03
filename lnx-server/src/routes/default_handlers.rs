use anyhow::Result;
use hyper::{Body, Request, Response};

use crate::error::LnxError;
use crate::responders::json_response;

pub async fn handle_404(_request: Request<Body>) -> Result<Response<Body>, LnxError> {
    json_response(404, "No route matched for path.")
}

pub async fn error_handler(err: routerify::RouteError) -> Response<Body> {
    handle_casting(err).await.unwrap_or_else(|e| {
        json_response(500, &e.to_string()).expect("serialize message")
    })
}

pub async fn handle_casting(err: routerify::RouteError) -> Result<Response<Body>> {
    let cast = match err.downcast::<LnxError>() {
        Ok(cast) => cast,
        Err(e) => {
            return json_response(500, &e.to_string()).map_err(anyhow::Error::from)
        },
    };

    let res = match *cast {
        LnxError::BadRequest(msg) => {
            json_response(400, msg).map_err(anyhow::Error::from)?
        },
        LnxError::UnAuthorized(msg) => {
            json_response(401, msg).map_err(anyhow::Error::from)?
        },
        LnxError::AbortRequest(resp) => resp,
        LnxError::Other(ref e) if e.source().is_some() => {
            json_response(500, &format!("error handling request: {}", e))
                .map_err(anyhow::Error::from)?
        },
        LnxError::Other(e) => {
            json_response(400, &e.to_string()).map_err(anyhow::Error::from)?
        },
        LnxError::ServerError(e) => {
            json_response(500, &format!("error handling request: {}", e))
                .map_err(anyhow::Error::from)?
        },
        LnxError::SerializationError(e) => {
            json_response(422, &e.to_string()).map_err(anyhow::Error::from)?
        },
    };

    Ok(res)
}
