use warp::{hyper::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub struct AccessDenied;
#[derive(Debug)]
pub struct WrongAction;
impl warp::reject::Reject for AccessDenied {}
impl warp::reject::Reject for WrongAction {}

pub async fn handle_reject(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let reply = if let Some(_e) = err.find::<AccessDenied>() {
        warp::reply::with_status("UNAUTHORIZED", StatusCode::UNAUTHORIZED)
    } else {
        eprintln!("Unhandled Rejection: {err:?}");
        warp::reply::with_status("INTERNAL_SERVER_ERROR", StatusCode::INTERNAL_SERVER_ERROR)
    };

    Ok(warp::reply::with_header(
        reply,
        "Access-Control-Allow-Origin",
        "*",
    ))
}
