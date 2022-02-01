//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{extract::{Path, Query}, routing::get, Router};
use std::net::SocketAddr;
use serde::{de, Deserialize, Deserializer};
use std::{fmt, str::FromStr};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    job_id: Option<i32>,
    tag: Option<String>,
}

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/matches/:match_id/sets/:set_id",
            get(get_match_stuff));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn get_match_stuff(
        Path((match_id, set_id)): Path<(i32, i32)>,
        Query(params): Query<Params>,
    ) -> String {
    format!("hello world: match = {}, set = {}, params = {:?}",
        match_id, set_id, params)
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

