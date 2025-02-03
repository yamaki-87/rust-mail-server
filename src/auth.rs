use rumbok::Data;

#[derive(Data)]
pub struct Auth {
    authenticated: bool,
    line: String,
}

const SUCCESS_MESSAGE_BYTES: &[u8] = b"235 Authentication successful\r\n";
const FAILED_MESSAGE_BYTES: &[u8] = b"535 Authentication failed\r\n";
const SEPERATOR_LENGTH: usize = 3;

impl Auth {
    /// 認証処理をしメッセージバイトを返す(成功か失敗かどうか)
    ///
    /// ## param
    /// * `input` 文字列
    ///
    /// ## return
    /// * メッセージバイト(成功か失敗かどうか)
    ///
    pub fn authenticated(&mut self, input: &str) -> &[u8] {
        // コマンド形式: AUTH PLAIN <base64_string> 3個に分割
        let parts: Vec<&str> = input.splitn(SEPERATOR_LENGTH, ' ').collect();

        if parts.len() < SEPERATOR_LENGTH {
            return b"501 Syntax: AUTH PLAIN <credentials>\r\n";
        } else {
            let encoded = parts[2];
            match base64::decode(encoded) {
                Ok(decoded) => {
                    // decoded のフォーマットは "\0user\0password"
                    let split: Vec<&[u8]> = decoded.split(|&b| b == 0).collect();
                    if split.len() == 3 && split[1] == b"user" && split[2] == b"password" {
                        self.authenticated = true;
                        return SUCCESS_MESSAGE_BYTES;
                    } else {
                        return FAILED_MESSAGE_BYTES;
                    }
                }
                Err(_) => {
                    return b"501 Invalid base64 encoding\r\n";
                }
            }
        }
    }
}
