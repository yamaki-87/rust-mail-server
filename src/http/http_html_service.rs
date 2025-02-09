use rumbok::Singleton;

use crate::EmailStore;

#[derive(Singleton)]
pub struct HtmlData {
    html: String,
}

impl HtmlData {
    pub fn init() {
        Self::initialize_instance(
            r#"<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Rust Mail</title>
  <style>
    /* リセット */
    * {
      margin: 0;
      padding: 0;
      box-sizing: border-box;
    }
    
    body {
      font-family: Arial, sans-serif;
      background-color: #f5f5f5;
      color: #333;
    }
    
    /* コンテナ：サイドバーとコンテンツ部分を横並びに */
    .container {
      display: flex;
      height: 100vh;
    }
    
    /* サイドバー：メール一覧 */
    .sidebar {
      width: 300px;
      background-color: #fff;
      border-right: 1px solid #ddd;
      overflow-y: auto;
    }
    
    /* 各メール項目 */
    .mail-item {
      padding: 15px;
      border-bottom: 1px solid #ddd;
      cursor: pointer;
      transition: background-color 0.2s ease;
    }
    
    .mail-item:hover {
      background-color: #f0f0f0;
    }
    
    .mail-item.active {
      background-color: #e0e0e0;
    }
    
    .mail-summary {
      font-size: 14px;
      font-weight: bold;
    }
    
    .mail-date {
      font-size: 12px;
      color: #999;
      margin-top: 5px;
    }
    
    /* メール詳細部分 */
    .content {
      flex: 1;
      padding: 20px;
      overflow-y: auto;
    }
    
    .header {
      margin-bottom: 20px;
      border-bottom: 1px solid #ddd;
      padding-bottom: 10px;
    }
    
    .subject {
      font-size: 20px;
      margin-bottom: 10px;
    }
    
    .sender {
      font-size: 14px;
      color: #555;
      margin-bottom: 10px;
    }

    .file {
      margin-bottom: 20px;
      border-bottom: 1px solid #ddd;
      padding-bottom: 10px;
      display: flex;
    }

    .file-item {
      align-items: center;
      justify-content: space-between;
      padding: 10px;
      margin: 5px 5px;
      border: 1px solid #ccc;
      border-radius: 8px;
      background-color: #f9f9f9;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
      width: 150px;
      text-overflow: ellipsis;
      overflow: hidden; 
    }
  
    .body {
      font-size: 16px;
      line-height: 1.5;
    }
    
    /* レスポンシブ対応：画面が狭い場合は縦並びに */
    @media (max-width: 768px) {
      .container {
        flex-direction: column;
      }
      .sidebar {
        width: 100%;
        max-height: 200px;
        border-right: none;
        border-bottom: 1px solid #ddd;
      }
    }
  </style>
</head>
<body>
  <div class="container">
    <!-- サイドバー：メール一覧 -->
    <div class="sidebar">
        $mail_list
    </div>
    
    <!-- メール詳細部分 -->
    <div class="content">
      <div class="header">
            $mail_header
      </div>
      <div class="file">
        $mail_file
      </div>
      <div class="body">
            $mail_body
      </div>
    </div>
  </div>
    <script>
      const ws = new WebSocket("ws://" + location.host + "/ws");
      ws.onmessage = function(event) {
        const data = event.data;
        const emailList = document.getElementById("email-list");
        const p = document.createElement("p");
        p.textContent = data;
        emailList.appendChild(p);
      };
    </script>
</body>
</html>
"#
            .into(),
        );
    }

    pub async fn create_mail_list_element(&self, emial_store: EmailStore) -> String {
        let store = emial_store.0.lock().await;
        let mut mail_list_element = "".to_string();
        let mut mail_header_element = "".to_string();
        let mut mail_file_item_element = "".to_string();
        let mut mail_body_element = "".to_string();
        for (i, email) in store.iter().enumerate() {
            mail_list_element.push_str(r#"<div class="mail-item">"#);
            mail_list_element.push_str(&format!(
                r#"<div class="mail-summary">{:?}</div>"#,
                email.get_subject()
            ));
            mail_list_element
                .push_str(&format!(r#"<div class="mail-date">2025-02-06 12:34</div>"#));
            mail_list_element.push_str("</div>");

            mail_header_element.push_str(&format!(
                r#"<div class="subject">{:?}</div>"#,
                email.get_subject()
            ));
            mail_header_element.push_str(&format!(
                r#"<div class="sender">{:?}</div>"#,
                email.get_from()
            ));

            for attachment in email.get_attachments() {
                if let Some(filename) = attachment.get_filename() {
                    mail_file_item_element
                        .push_str(&format!(r#"<div class="file-item">{}</div>"#, filename));
                }
            }

            mail_body_element.push_str(&format!(
                "<p>{}</p>",
                htmlescape::encode_minimal(&email.get_body())
            ));
        }

        let html_content_clone = self.html.clone();
        html_content_clone
            .replace("$mail_list", &mail_list_element)
            .replace("$mail_header", &mail_header_element)
            .replace("$mail_file", &mail_file_item_element)
            .replace("$mail_body", &mail_body_element)
    }

    pub fn get_html_content(&self) -> &str {
        &self.html
    }
}
