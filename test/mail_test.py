import smtplib
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText
from email.mime.base import MIMEBase
from email import encoders
import os
import argparse


smtp_server = "localhost"
port = 2525

parser = argparse.ArgumentParser(description="コマンドライン引数で処理変更")

parser.add_argument("-t","--html",action="store_true",help="html形式でメールを送る")
parser.add_argument("-s","--tls",action="store_true",help="TLS通信でメールを送る")

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

def create_message(f,t,s,b,h="plain"):
    msg = MIMEMultipart() 
    msg["From"]  = f
    msg["To"] = t
    msg["Subject"] = s
    msg.attach(MIMEText(b,h,'utf-8'))

    return msg

def send_email_with_tls():
    from_addr = 'sender@TLSexample.com'
    to_addr = 'recipient@TLSexample.com'
    subject = '件名TLS TEST'
    body = 'これは TLS を使用した安全なメール送信のサンプルです。'
    msg = create_message(from_addr,to_addr,subject,body).as_string()
    try:
        # SMTP サーバーに接続し、TLS を開始
        with smtplib.SMTP(smtp_server, port) as server:
            server.set_debuglevel(1)
            server.ehlo()           # サーバーとの通信を初期化
            server.starttls()       # TLS で通信を暗号化
            server.ehlo()           # TLS 開始後に再度初期化
            server.login("hoge","hageS")
            server.sendmail(from_addr,to_addr,msg)
            print("メールが送信されました！")
    except Exception as e:
        import traceback
        traceback.print_exc()

file_paths = ['file.txt', 'タイトルなし.png']

def main():
    args = parser.parse_args()
    if args.html:
        html_content = """\
<html>
  <body>
    <h2>こんにちは！</h2>
    <p>このメールは <b>HTML形式</b> です。</p>
    <p style="color: red;">これは赤いテキストです。</p>
    <a href="https://example.com">リンクはこちら</a>
  </body>
</html>
"""
        try:
            with smtplib.SMTP(smtp_server,port) as server:
                server.sendmail(sender_email,recipient_email,create_message(sender_email,recipient_email,"test",html_content,"html").as_string())
            print("メールが送信されました！")
        except Exception as e:
            print(f"エラーが発生しました: {e}")
    elif args.tls:
        send_email_with_tls()
    else:
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
                server.set_debuglevel(1)
                server.sendmail(sender_email, recipient_email, message.as_string())
                server.sendmail("test@example.com","saitama.sf@example.com",create_message("test@example.com","saitama.sf@example.com","明日の会議について","明日の会議はなしで").as_string())
                server.sendmail("ibariaki@example.com","iba.sf@example.com",create_message("ibariaki@example.com","iba.sf@example.com","欠席連絡","明後日から欠席します").as_string())
            print("メールが送信されました！")
        except Exception as e:
            print(f"エラーが発生しました: {e}")

if __name__ == '__main__':
    main()