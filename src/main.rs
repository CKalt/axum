use axum::{response::Html, routing::get, Router};
use std::net::SocketAddr;

use std::fs::File;
use daemonize::Daemonize;

use tokio::runtime::Runtime;

fn main() {
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();


    let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        .working_directory("/tmp") // for default behaviour.
        .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr)  // Redirect stderr to `/tmp/daemon.err`.
        .privileged_action(|| "Executed before drop privileges");


    match daemonize.start() {
        Ok(_) => {
            let rt  = Runtime::new().unwrap();

            rt.block_on(async {
                // build our application with a route
                let app = Router::new().route("/", get(handler));

                // run it
                let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
                println!("listening on {}", addr);
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();

                println!("Success, daemonized");
            });
        },
        Err(e) => eprintln!("Error, {}", e),
    }
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
