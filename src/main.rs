use std::sync::Arc;

use anyhow::Result;
use env_logger::Builder;
use http_server::run_http_server;
use smtp_server::run_stmp_server;
use tokio::sync::Mutex;
// https://qiita.com/simonritchie/items/87d3743e138763ff3e85
mod auth;
mod command;
mod constants;
mod email;
mod http_server;
mod mail_io;
mod smtp_server;
/// 共通のメールアドレスの型
#[derive(Clone)]
struct EmailStore(Arc<Mutex<Vec<String>>>);

#[tokio::main]
async fn main() -> Result<()> {
    logger_init();

    // 受信メール保存する共通ストア(メモリー上)
    let email_store = EmailStore(Arc::new(Mutex::new(Vec::new())));

    // SMTP サーバー（ポート 2525）を起動
    let smtp_sore = email_store.clone();
    let smtp_server = tokio::spawn(async move { run_stmp_server(smtp_sore).await });

    // HTTP サーバー（ポート 8025）を起動（Web UI 用）
    let http_store = email_store.clone();
    let http_server = tokio::spawn(async move { run_http_server(http_store).await });

    // 両方のサーバーが動作するのを待機
    smtp_server.await??;
    http_server.await??;
    Ok(())
}

/// logger init処理
fn logger_init() {
    let log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    Builder::new().filter_level(log_level).init();
}
