//! Run with
//!
//! ```not_rust
//! cargo run -p example-hello-world
//! ```

use axum::{
    AddExtensionLayer,
    extract::Extension,
    extract::{Path, Query, Json},
    routing::{get, put}, Router
};
use std::net::SocketAddr;
use serde::{de, Serialize, Deserialize, Deserializer};
use std::{fmt, str::FromStr, sync::Arc};
use serde_json::json;

struct AppState {
    cfg: String,
    num: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigUpdate {
    hub_id: String,
    arm_parameters: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    job_id: Option<i32>,
    tag: Option<String>,
}

use std::fs::File;
use daemonize::Daemonize;
use tokio::runtime::Runtime;

fn main() {
    let shared_state = Arc::new(AppState {
        cfg: String::from("hello theo"),
        num: 837,
    });


    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr)  // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => {
            let rt  = Runtime::new().unwrap();
            rt.block_on(async {
                // build our application with a route
                let app = Router::new()
                        .route("/config", put(put_update_config))
                        .route("/matches/:match_id/sets/:set_id", get(get_match_stuff))
                        .layer(AddExtensionLayer::new(shared_state));

                // run it
                let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
                println!("listening on {}", addr);
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            });

            println!("Success, daemonized");
        }
        Err(e) => eprintln!("Error, {}", e),
    }
}

async fn get_match_stuff(
        Extension(app_state): Extension<Arc<AppState>>,
        Path((match_id, set_id)): Path<(i32, i32)>,
        Query(params): Query<Params>,
    ) -> String {

    assert_eq!(tokio::spawn(async { 1 }).await.unwrap(), 1);

    format!("app_state.cfg = {}, app_state.num = {}\n\
        hello world: match = {}, set = {}, params = {:?}",
            app_state.cfg, app_state.num, match_id, set_id, params)
}

async fn put_update_config(
        Extension(app_state): Extension<Arc<AppState>>,
        opt_cfg_upd: Option<Json<ConfigUpdate>>
    ) -> String {
    assert_eq!(tokio::spawn(async { 1 }).await.unwrap(), 1);

    let json = 
        if let Some(cfg_upd) = opt_cfg_upd {
            json!(
                {
                    "result" : "success",
                    "data" : {
                        "cfg_upd": {
                            "hub_id":         cfg_upd.hub_id,
                            "arm_parameters": cfg_upd.arm_parameters
                        },
                        "app_state": {
                            "cfg": app_state.cfg,
                            "num": app_state.num,
                        }
                    }
                }
            )
        } else {
            json!(
                {
                    "result" : "error",
                    "data"   : "body not json ConfigUpdate Object"
                }
            )
        };
    serde_json::to_string_pretty(&json).unwrap()
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

