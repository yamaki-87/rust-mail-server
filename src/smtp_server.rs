use std::io::ErrorKind;

use anyhow::Result;
use chrono::Local;
use log::{error, info, warn};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
    sync::broadcast::Sender,
};
use tokio_rustls::{server::TlsStream, TlsAcceptor};

use crate::{
    auth::Auth,
    command::{self, Command, WebSocketCommand},
    constants::*,
    email::EmailData,
    EmailStore,
};

pub async fn run_stmp_server(
    email_store: EmailStore,
    ws_tx: Sender<String>,
    acceptor: Option<TlsAcceptor>,
) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:2525").await?;
    info!("SMTP Server is running on 127.0.0.1:2525 ...");

    loop {
        // 新しい接続を受け付ける
        let (socket, addr) = listener.accept().await?;
        info!("新しい接続先: {}", addr);

        let store = email_store.clone();
        let ws_tx_clone = ws_tx.clone();
        let accptor_clone = acceptor.clone();
        // 接続ごとに別タスクで処理
        tokio::spawn(async move {
            if let Err(e) = process_connection(socket, store, ws_tx_clone, accptor_clone).await {
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
    acceptor: Option<TlsAcceptor>,
) -> Result<()> {
    // 読み書き用にストリームを分割
    //let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(socket);

    // クライアントに対して SMTP の挨拶を送信
    reader
        .get_mut()
        .write_all(b"220 Rust SMTP Server Ready\r\n")
        .await?;

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

        let command = line.trim_end().to_uppercase();
        match command::get_command_from_str(&command) {
            Command::Helo => {
                info!("[HELO] command is {}", &command);
                reader.get_mut().write_all(b"250 Hello\r\n").await?;
            }
            Command::Ehlo => {
                info!("[EHLO] command is {}", &command);
                reader
                    .get_mut()
                    .write_all(b"250-MyRustSMTP\r\n250-STARTTLS\r\n250 AUTH PLAIN LOGIN\r\n")
                    .await?;
            }
            Command::StartTls => {
                if let Some(acceptor) = acceptor.clone() {
                    info!("STARTTLS -> TLSへ切り替え成功");
                    reader.get_mut().write_all(STARTTLS_MESSAGE_BYTES).await?;
                    let s = reader.into_inner();
                    let tls_socket = acceptor.accept(s).await?;
                    handle_tls_client(tls_socket, email_store, ws_tx).await?;
                    break;
                } else {
                    warn!("STARTTLSをサポートしていません");
                    reader
                        .get_mut()
                        .write_all(STARTTLS_NO_SUPPORTED_MESSAGE_BYTES)
                        .await?;
                }
            }
            Command::MailFrom => {
                reader.get_mut().write_all(OK_MESSAGE_BYTES).await?;
            }
            Command::RcptTo => {
                reader.get_mut().write_all(OK_MESSAGE_BYTES).await?;
            }
            Command::Data => {
                reader
                    .get_mut()
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

                reader.get_mut().write_all(b"250 Ok:queued\r\n").await?;
            }
            Command::Quit => {
                reader.get_mut().write_all(b"221 Bye\r\n").await?;
                break;
            }
            Command::AuthPlain => {
                // AUTH PLAIN AHVzZXIAcGFzc3dvcmQ=   <-- 「\0user\0password」を base64 エンコードした文字列
                let message_bytes = auth.authenticated(&command);
                reader.get_mut().write_all(message_bytes).await?;
            }
            Command::Unknown => {
                reader
                    .get_mut()
                    .write_all(b"500 Unrecongnized command\r\n")
                    .await?;
            }
        }
    }

    Ok(())
}

async fn handle_tls_client(
    socket: TlsStream<tokio::net::TcpStream>,
    email_store: EmailStore,
    ws_tx: Sender<String>,
) -> Result<()> {
    let (mut reader, mut writer) = tokio::io::split(socket);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    send_tls(&mut writer, b"220 Rust SMTP(TLS) Server Ready\r\n").await?;
    loop {
        line.clear();
        let n = read_tls(&mut reader, &mut line).await?;
        if n == 0 {
            break;
        }

        let command = line.trim_end().to_uppercase();
        match command::get_command_from_str(&command) {
            Command::Helo | Command::Ehlo => {
                info!("[HELO EHLO] command is {}", &command);
                //writer.write_all(b"250 Hello\r\n").await?;
                send_tls(
                    &mut writer,
                    b"250 MyRustSMTP (TLS)\r\n250 AUTH PLAIN LOGIN\r\n",
                )
                .await?;
            }
            Command::StartTls => {
                warn!("既にTLS通信です");
            }
            Command::MailFrom => {
                send_tls(&mut writer, OK_MESSAGE_BYTES).await?;
            }
            Command::RcptTo => {
                send_tls(&mut writer, OK_MESSAGE_BYTES).await?;
            }
            Command::Data => {
                send_tls(&mut writer, b"354 End data with <CR><LF>.<CR><LF>\r\n").await?;

                let mut datas = vec![];
                loop {
                    line.clear();
                    let bytes_read = read_tls(&mut reader, &mut line).await?;
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

                send_tls(&mut writer, b"250 Ok:queued\r\n").await?;
            }
            Command::Quit => {
                send_tls(&mut writer, b"221 Bye\r\n").await?;
                break;
            }
            Command::AuthPlain => {
                // // AUTH PLAIN AHVzZXIAcGFzc3dvcmQ=   <-- 「\0user\0password」を base64 エンコードした文字列
                // let message_bytes = auth.authenticated(&command);
                // writer.write_all(message_bytes).await?;
            }
            Command::Unknown => {
                send_tls(&mut writer, b"500 Unrecongnized command\r\n").await?;
            }
        }
    }
    Ok(())
}

/// ## Summary
/// tlsで通信してメールを送る
///
/// ## Note
/// ErrorKind::UnexpectedEofはrustlsだとERROR扱いになる
/// 相手が close_notify を送らずに接続を閉じた時に起きるerror
/// 一応このメールサーバーではclose_notifyがなくても正常終了とみなします。
///
/// ## Parameters
/// - `writer`:
/// - `msg_byte`:
///
/// ## Returns
///
/// ## Examples
///```
///
///```
async fn send_tls(writer: &mut WriteHalf<TlsStream<TcpStream>>, msg_byte: &[u8]) -> Result<()> {
    if let Err(e) = writer.write_all(msg_byte).await {
        if e.kind() == ErrorKind::UnexpectedEof {
            // TLS的には “close_notify” が来ていないが、
            // こちらとしては「相手が接続を閉じただけ」として扱う。
            warn!("{}", e);
            return Ok(());
        }
        return Err(e.into());
    }
    Ok(())
}

async fn read_tls(
    reader: &mut BufReader<ReadHalf<TlsStream<TcpStream>>>,
    line: &mut String,
) -> Result<usize> {
    match reader.read_line(line).await {
        Ok(result) => Ok(result),
        Err(e) => {
            if e.kind() == ErrorKind::UnexpectedEof {
                // TLS的には “close_notify” が来ていないが、
                // こちらとしては「相手が接続を閉じただけ」として扱う。
                warn!("{}", e);
                return Ok(0);
            }
            return Err(e.into());
        }
    }
}
