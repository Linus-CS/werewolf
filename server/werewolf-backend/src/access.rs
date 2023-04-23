use crate::rejection::AccessDenied;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection};

const ACCESS_TOKEN: &str = "1235";
const MASTER_TOKEN: &str = "5321";

#[derive(Debug, Serialize, Deserialize)]
struct AccessQuery {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct MasterQuery {
    master_token: String,
}

pub fn check_access<F, T>(
    filter: F,
) -> impl Filter<Extract = (T,), Error = Rejection> + Clone + Send + Sync
where
    F: Filter<Extract = (T,), Error = Rejection> + Clone + Send + Sync,
    F::Extract: warp::Reply,
{
    let check = warp::query::<AccessQuery>()
        .and_then(|query: AccessQuery| async move {
            if query.access_token == ACCESS_TOKEN {
                Ok(())
            } else {
                Err(warp::reject::custom(AccessDenied))
            }
        })
        .untuple_one();

    check.and(filter)
}

pub fn check_master<F, T>(
    filter: F,
) -> impl Filter<Extract = (T,), Error = Rejection> + Clone + Send + Sync
where
    F: Filter<Extract = (T,), Error = Rejection> + Clone + Send + Sync,
    F::Extract: warp::Reply,
{
    let check = warp::query::<MasterQuery>()
        .and_then(|query: MasterQuery| async move {
            if query.master_token == MASTER_TOKEN {
                Ok(())
            } else {
                Err(warp::reject::custom(AccessDenied))
            }
        })
        .untuple_one();

    check.and(filter)
}
