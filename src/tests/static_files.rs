use std::net::SocketAddr;
use std::time::Duration;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use reqwest::Url;
use tokio::spawn;
use tokio::task::spawn_blocking;
use crate::apis::DownloadApiTrait;
use crate::apis::static_files::{StaticFileAsset, StaticFileRelease};

async fn init_demo_http_server(port: u16) {
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/my_endpoint.json", get(endpoint_json));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("demo static file server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn endpoint_json() -> impl IntoResponse {
    let demo_release = StaticFileRelease {
        version: "1.0.0".to_string(),
        assets: vec![
            StaticFileAsset {
                name: "update".to_string(),
                url: "https://cdn.discordapp.com/attachments/845321775302836254/1009852069307502652/IMG_20220818_175052.jpg".to_string()
            }
        ]
    };
    (StatusCode::CREATED, Json(vec![demo_release]))
}

#[tokio::test]
pub async fn test_static_files() {
    let port = portpicker::pick_unused_port().expect("No ports free");
    spawn(async move {
        init_demo_http_server(port).await;
    });
    tokio::time::sleep(Duration::from_secs(5)).await;

    spawn_blocking(move || {
        let mut api = crate::apis::static_files::StaticFilesApi::new(Url::parse(&format!("http://localhost:{}/my_endpoint.json", port)).unwrap());
        let test_version = "0.1.0";
        api.current_version(test_version);

        let download = api.get_newer(&None).unwrap();
        println!("download: {:?} current ver: {}", download, test_version);

        if let Some(download) = download {
            api.download(
                &download.assets[0],
                None
            ).unwrap();
        }
    });
}