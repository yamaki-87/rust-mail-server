import smtplib
from email.mime.text import MIMEText
from email.mime.multipart import MIMEMultipart


smtp_server = "localhost"
port = 2525

sender_email = "sender@example.com"
recipient_email = "recipient@example.com" 
subject = "ローカルSMTPサーバーからのテストメール"
body = "これはローカルSMTPサーバーから送信したテストメールです。"

# メールの作成
message = MIMEMultipart()
message["From"] = sender_email
message["To"] = recipient_email
message["Subject"] = subject

message.attach(MIMEText(body, "plain"))

# SMTPサーバーに接続してメールを送信
try:
    # ローカルSMTPサーバーに接続（認証なし）
    server = smtplib.SMTP(smtp_server, port)
    
    # メールを送信
    server.sendmail(sender_email, recipient_email, message.as_string())
    print("メールが送信されました！")

except Exception as e:
    print(f"エラーが発生しました: {e}")

finally:
    server.quit()