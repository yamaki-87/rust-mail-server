use crate::util::base64;
use log::{error, info};
use rumbok::Data;

#[derive(Data)]
pub struct Auth {
    authenticated: bool,
    username: String,
    password: String,
}

const SUCCESS_MESSAGE_BYTES: &[u8] = b"235 Authentication successful\r\n";
const FAILED_MESSAGE_BYTES: &[u8] = b"535 Authentication failed\r\n";
const SEPERATOR_LENGTH: usize = 3;

impl Auth {
    pub fn parse_plain_credentials(&mut self, input: &str) -> &[u8] {
        // コマンド形式: AUTH PLAIN <base64_string> 3個に分割
        let parts: Vec<&str> = input.splitn(SEPERATOR_LENGTH, ' ').collect();
        if parts.len() < SEPERATOR_LENGTH {
            return b"501 Syntax: AUTH PLAIN <credentials>\r\n";
        }

        let bytes = match base64::deocde_bytes(parts[2]) {
            Ok(b) => b,
            Err(e) => {
                error!("{}", e);
                return b"501 Invalid base64 encoding\r\n";
            }
        };

        let split: Vec<&[u8]> = bytes.split(|&b| b == 0).collect();
        if split.len() < SEPERATOR_LENGTH {
            return FAILED_MESSAGE_BYTES;
        }

        let username = String::from_utf8_lossy(split[1]).to_string();
        let password = String::from_utf8_lossy(split[2]).to_string();

        if username.is_empty() {
            return b"501 Missing credentials";
        }
        info!("username:{} password:{}", &username, &password);

        self.username = username;
        self.password = password;
        self.authenticated = true;

        return SUCCESS_MESSAGE_BYTES;
    }
}
