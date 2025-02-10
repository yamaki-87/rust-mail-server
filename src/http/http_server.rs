use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use tokio::sync::broadcast;
use warp::Filter;

use crate::{
    email::{EmailSummary, SearchQuery},
    EmailStore,
};

use super::http_html_service::{self, HtmlData};

/// HTTP サーバーを起動して、受信メールを Web 画面で表示する関数
pub async fn run_http_server(
    email_store: EmailStore,
    ws_tx: broadcast::Sender<String>,
) -> Result<()> {
    // email_store を各リクエストで利用できるようにする
    let store_filter = warp::any().map(move || email_store.clone());
    // ルートパスにアクセスしたときのハンドラ
    let index = warp::path::end()
        .and(store_filter.clone())
        .and_then(handle_index);

    // API: GET /api/emails → すべてのメールを JSON で返す
    let api_email = warp::path!("api" / "emails")
        .and(warp::get())
        .and(warp::query::<SearchQuery>())
        .and(store_filter.clone())
        .and_then(handle_api_emails_get);

    let api_email_detail = warp::path!("api" / "emails" / usize)
        .and(warp::get())
        .and(store_filter.clone())
        .and_then(handle_api_emails_detail);

    // API: DELETE /api/emails/{id} → 指定 id のメールを削除
    let api_email_delete = warp::path!("api" / "emails" / usize)
        .and(warp::delete())
        .and(store_filter.clone())
        .and_then(handle_api_email_delete);

    // API: POST /api/emails/clear → すべてのメールをクリア（テスト用）
    let api_emails_clear = warp::path!("api" / "emails" / "clear")
        .and(warp::post())
        .and(store_filter.clone())
        .and_then(handle_api_delete_batch);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_ws_tx(ws_tx.clone()))
        .map(|ws: warp::ws::Ws, ws_tx: broadcast::Sender<String>| {
            ws.on_upgrade(move |socket| handle_ws_connection(socket, ws_tx))
        });
    
    let cors = warp::cors()
    .allow_any_origin()
    .allow_methods(vec!["GET","POST","DELETE","PUT"])
    .build();


    // 全てのrouteをまとめる
    let routes = index
        .or(api_email)
        .or(api_email_delete)
        .or(api_email_detail)
        .or(api_emails_clear)
        .or(ws_route)
        .with(cors);

    // ポート 8025 で HTTP サーバーを起動
    warp::serve(routes).run(([127, 0, 0, 1], 8025)).await;

    info!("Http Server(Web UI) running on 127.0.0.1:8025 ...");
    Ok(())
}
/// Web UI のルートハンドラ：受信メール一覧を HTML で返す
async fn handle_index(email_store: EmailStore) -> Result<impl warp::Reply, warp::Rejection> {
    match tokio::fs::read_to_string("static/index.html").await {
        Ok(contents) => Ok(warp::reply::html(http_html_service::init_html(email_store,contents).await)) ,
        Err(e) => {
            error!("handle_index error: {}",e);
            Ok(warp::reply::html("<h1>Files not found</h1>".to_string()))
        }
    }
}

/// API ハンドラ：GET /api/emails → すべてのメールを JSON で返す
async fn handle_api_emails_get(
    search_query: SearchQuery,
    email_store: EmailStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let store = email_store.0.lock().await;
    let emails: Vec<EmailSummary> = store
        .iter()
        .enumerate()
        .filter(|(_, email)| {
            if let Some(query) = search_query.get_q() {
                let query_lower = query.to_lowercase();
                return email
                    .get_subject()
                    .as_ref()
                    .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                    || email
                        .get_from()
                        .as_ref()
                        .map_or(false, |s| s.to_lowercase().contains(&query_lower))
                    || email.get_body().to_lowercase().contains(&query_lower);
            }
            true
        })
        .map(|(i, email)| email.convert_to_email_summary(i))
        .collect();

    Ok(warp::reply::json(&emails))
}

/// API ハンドラ：GET /api/emails/{id} → 指定したメールの詳細を返す
async fn handle_api_emails_detail(
    id: usize,
    email_store: EmailStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let store = email_store.0.lock().await;
    if let Some(email) = store.get(id) {
        let email_summary = email.convert_to_email_summary(id);
        Ok(warp::reply::json(&email_summary))
    } else {
        Err(warp::reject::not_found())
    }
}

/// API ハンドラ：DELETE /api/emails/{id} → 指定したメールを削除する
async fn handle_api_email_delete(
    id: usize,
    email_store: EmailStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut store = email_store.0.lock().await;
    if id < store.len() {
        store.remove(id);
        Ok(warp::reply::with_status(
            "Delete",
            warp::http::StatusCode::OK,
        ))
    } else {
        Err(warp::reject::not_found())
    }
}

/// API ハンドラ：POST /api/emails/clear → すべてのメールをクリアする
async fn handle_api_delete_batch(
    email_store: EmailStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut store = email_store.0.lock().await;
    store.clear();
    Ok(warp::reply::with_status(
        "Clean",
        warp::http::StatusCode::OK,
    ))
}

fn with_ws_tx(
    ws_tx: broadcast::Sender<String>,
) -> impl Filter<Extract = (broadcast::Sender<String>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || ws_tx.clone())
}

async fn handle_ws_connection(
    ws: warp::ws::WebSocket,
    ws_tx: tokio::sync::broadcast::Sender<String>,
) {
    let mut rx = ws_tx.subscribe();
    let (mut ws_tx_sink, _ws_rx) = ws.split();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            info!("websocket recv: {}", &msg);
            if ws_tx_sink.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });
}
