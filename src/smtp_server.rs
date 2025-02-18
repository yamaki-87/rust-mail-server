use anyhow::Result;
use base64::engine::general_purpose::NO_PAD;
use chrono::Local;
use log::{error, info};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast::Sender,
};

use crate::{
    auth::Auth,
    command::{self, Command, WebSocketCommand},
    constants::*,
    email::EmailData,
    EmailStore,
};

pub async fn run_stmp_server(email_store: EmailStore, ws_tx: Sender<String>) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2525").await?;
    info!("SMTP Server is running on 127.0.0.1:2525 ...");

    loop {
        // 新しい接続を受け付ける
        let (socket, addr) = listener.accept().await?;
        info!("新しい接続先: {}", addr);

        let store = email_store.clone();
        let ws_tx_clone = ws_tx.clone();
        // 接続ごとに別タスクで処理
        tokio::spawn(async move {
            if let Err(e) = process_connection(socket, store, ws_tx_clone).await {
                error!("Error: {}", e);
            }
        });
    }
}

/// 1 つの SMTP 接続を処理する関数
async fn process_connection(
    socket: TcpStream,
    email_store: EmailStore,
    ws_tx: Sender<String>,
) -> Result<()> {
    // 読み書き用にストリームを分割
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    // クライアントに対して SMTP の挨拶を送信
    writer.write_all(b"220 Rust SMTP Server Ready\r\n").await?;

    // 認証状態を保持する
    let mut auth = Auth::default();
    let mut line = String::new();

    loop {
        line.clear();

        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            // 接続がcloseされた場合
            break;
        }

        let trimmed = line.trim_end();
        info!("Received: {}", trimmed);

        match command::get_command_from_str(&trimmed.to_uppercase()) {
            Command::Helo | Command::Ehlo => {
                writer.write_all(b"250 Hello\r\n").await?;
            }
            Command::MailFrom => {
                writer.write_all(OK_MESSAGE_BYTES).await?;
            }
            Command::RcptTo => {
                writer.write_all(OK_MESSAGE_BYTES).await?;
            }
            Command::Data => {
                writer
                    .write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n")
                    .await?;

                let mut datas = vec![];
                loop {
                    line.clear();
                    let bytes_read = reader.read_line(&mut line).await?;
                    if bytes_read == 0 {
                        // クライアントが切断
                        break;
                    }

                    if line.trim_end() == "." {
                        // 行にドットのみならデータ終了
                        break;
                    }

                    datas.push(line.clone());
                }

                let email_content = datas.join("");
                let mail_data = EmailData::new(email_content, Local::now());
                // mutexをすぐ解放するための処置
                {
                    // 受信したメールを共有ストアに保存
                    let mut store = email_store.0.lock().await;
                    store.push(mail_data.clone());
                }
                // WebSocket 用に新着メール通知を送信
                let _ = ws_tx.send(WebSocketCommand::Update.into());

                writer.write_all(b"250 Ok:queued\r\n").await?;
            }
            Command::Quit => {
                writer.write_all(b"221 Bye\r\n").await?;
                break;
            }
            Command::AuthPlain => {
                // AUTH PLAIN AHVzZXIAcGFzc3dvcmQ=   <-- 「\0user\0password」を base64 エンコードした文字列
                let message_bytes = auth.authenticated(trimmed);
                writer.write_all(message_bytes).await?;
            }
            Command::Unknown => {
                writer.write_all(b"500 Unrecongnized command\r\n").await?;
            }
        }
    }

    Ok(())
}
