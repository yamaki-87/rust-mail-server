use anyhow::Result;
use log::info;
use warp::Filter;

use crate::{email::Email, EmailStore};

/// HTTP サーバーを起動して、受信メールを Web 画面で表示する関数
pub async fn run_http_server(email_store: EmailStore) -> Result<()> {
    // email_store を各リクエストで利用できるようにする
    let store_filter = warp::any().map(move || email_store.clone());
    // ルートパスにアクセスしたときのハンドラ
    let index = warp::path::end()
        .and(store_filter.clone())
        .and_then(handle_index);

    // API: GET /api/emails → すべてのメールを JSON で返す
    let api_email = warp::path!("api" / "emails")
        .and(warp::get())
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

    // 全てのrouteをまとめる
    let routes = index
        .or(api_email)
        .or(api_email_delete)
        .or(api_email_detail)
        .or(api_emails_clear);

    // ポート 8025 で HTTP サーバーを起動
    warp::serve(routes).run(([127, 0, 0, 1], 8025)).await;

    info!("Http Server(Web UI) running on 127.0.0.1:8025 ...");
    Ok(())
}
/// Web UI のルートハンドラ：受信メール一覧を HTML で返す
async fn handle_index(email_store: EmailStore) -> Result<impl warp::Reply, warp::Rejection> {
    let store = email_store.0.lock().await;
    let mut html = String::new();
    html.push_str(
        "<html><head><meta charset=\"utf-8\"><title>Local Mail Server</title></head><body>",
    );
    html.push_str("<h1>受信メール一覧</h1>");
    if store.is_empty() {
        html.push_str("<p>メールはまだ受信されていません。</p>");
    } else {
        for (i, email) in store.iter().enumerate() {
            // ※ htmlescape で HTML エスケープして表示
            html.push_str(&format!(
                "<h2>メール {}:</h2><pre>{}</pre><hr>",
                i + 1,
                htmlescape::encode_minimal(email)
            ));
        }
    }
    html.push_str("</body></html>");
    Ok(warp::reply::html(html))
}

/// API ハンドラ：GET /api/emails → すべてのメールを JSON で返す
async fn handle_api_emails_get(
    email_store: EmailStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let store = email_store.0.lock().await;
    let emails: Vec<Email> = store
        .iter()
        .enumerate()
        .map(|(i, content)| Email::new_all(i, content.clone()))
        .collect();

    Ok(warp::reply::json(&emails))
}

/// API ハンドラ：GET /api/emails/{id} → 指定したメールの詳細を返す
async fn handle_api_emails_detail(
    id: usize,
    email_store: EmailStore,
) -> Result<impl warp::Reply, warp::Rejection> {
    let store = email_store.0.lock().await;
    if let Some(content) = store.get(id) {
        let email = Email::new_all(id, content.clone());
        Ok(warp::reply::json(&email))
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
