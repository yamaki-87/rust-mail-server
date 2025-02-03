use anyhow::Result;
use chrono::Utc;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

/// 受信したメールを保存
///
/// ## param
///
/// * `datas` - 受け取った文字列
///
/// ## return
/// 成功したかどうか(Result)
pub async fn save_data(datas: &Vec<String>) -> Result<()> {
    // 受信メールを保存処理
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("./mails/email.txt")
        .await?;

    let header = format!("---- New Email at {} ----", Utc::now());
    file.write_all(header.as_bytes()).await?;
    for data in datas {
        file.write_all(data.as_bytes()).await?;
    }

    Ok(())
}
