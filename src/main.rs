use axum::{
    response::Html,
    routing::get,
    Router,
    AddExtensionLayer,
    extract::{Extension, Path},
};
use tokio_postgres::{Client, NoTls};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use serde_json::Value;
use serde_json::Value::Array;

#[derive(Debug, Clone)]
struct Config {
    db_connect_str: String,
}

struct AppState {
    cfg: Mutex<Config>,
}

fn main() {
    let cfg = Config {
        db_connect_str: String::from(
    "connect_str=postgres://chris:hello@localhost:5432/sports3d"),
    };
    let cfg_data = Arc::new(AppState{
            cfg: Mutex::new(cfg.clone()),
    });
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let app = Router::new()
    .route("/", get(handler))
    .route("/jobs/:jobid/log-responses", get(get_logs_for_job))
    .layer(AddExtensionLayer::new(cfg_data));

        // run it
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        println!("listening on {}", addr);
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}

async fn get_logs_for_job(
        Extension(data): Extension<Arc<AppState>>,
        Path(job_id): Path<i32>,
    ) -> String {
    let cfg = data.cfg.lock().unwrap();
    let (client, connection) =
        match tokio_postgres::connect(&cfg.db_connect_str, NoTls).await {
            Ok(tup) => tup,
            Err(_) => {
                return String::from("db connect fail");
            },
        };

    /*
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    */

    if let Err(e) = connection.await {
        panic!(format!("db_connect failure e={}", e));
    }
        
    fetch_log_rows(&client, job_id).await
}

async fn fetch_log_rows(client: &Client, job_id: i32) -> String {
    let query = r#"
        SELECT logresponse
          FROM p3d_ds_log
         WHERE logresponse IS NOT NULL
           AND jobid = {}
         ORDER by logid"#;

    let rows = 
        match client.query(query, &[&job_id]).await {
            Ok(r) => r,
            Err(e) => panic!(format!("query for fetch_log_rows failed e={}", e)),
        };

    let mut results: Vec<Value> = Vec::new();
    for row in rows.iter() {
        let rsp_json: Value = row.get(0);
        results.push(rsp_json);
    }

    let json_result = Array(results);
    serde_json::to_string(&json_result).unwrap()
}

