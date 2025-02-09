import smtplib
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.base import MIMEBase
from email import encoders
import os


smtp_server = "localhost"
port = 2525

sender_email = "sender@example.com"
recipient_email = "recipient@example.com" 
subject = "ローカルSMTPサーバーからのテストメール"
body = "これはローカルSMTPサーバーから送信したテストメールです。BODY"

# メールの作成
message = MIMEMultipart()
message["From"] = sender_email
message["To"] = recipient_email
message["Subject"] = subject

message.attach(MIMEText(body, "plain"))

file_paths = ['file.txt', 'タイトルなし.png']

for file_path in file_paths:
    if os.path.exists(file_path):
        with open(file_path,'rb') as f:
            mime_part = MIMEBase('application', 'octet-stream')
            mime_part.set_payload(f.read())
            encoders.encode_base64(mime_part)
            mime_part.add_header('Content-Disposition', f'attachment; filename="{os.path.basename(file_path)}"')
            message.attach(mime_part)
    else:
        print(f"ファイルが見つかりません: {file_path}")
# SMTPサーバーに接続してメールを送信
try:
    # ローカルSMTPサーバーに接続（認証なし）
    with smtplib.SMTP(smtp_server, port) as server:
        # メールを送信
        server.sendmail(sender_email, recipient_email, message.as_string())
    print("メールが送信されました！")
except Exception as e:
    print(f"エラーが発生しました: {e}")