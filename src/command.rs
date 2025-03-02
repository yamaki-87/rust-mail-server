use log::debug;

use crate::constants::*;

pub enum Command {
    Helo,
    Ehlo,
    MailFrom,
    RcptTo,
    Data,
    Quit,
    AuthPlain,
    AuthLogin,
    StartTls,
    Unknown,
}

pub enum WebSocketCommand {
    Update,
}

impl Into<String> for WebSocketCommand {
    fn into(self) -> String {
        match self {
            WebSocketCommand::Update => "UPDATE".into(),
        }
    }
}

/// 文字列からコマンドを取得
///
/// ## param
///
/// * input 文字列
///
/// ## return
///
/// * Command
pub fn get_command_from_str(input: &str) -> Command {
    use Command::*;

    if input.starts_with(HELO) {
        debug!("command is {}", input);
        return Helo;
    } else if input.starts_with(EHLO) {
        debug!("command is {}", input);
        return Ehlo;
    } else if input.starts_with(MAILFROM) {
        debug!("command is {}", input);
        return MailFrom;
    } else if input.starts_with(RCPTTO) {
        debug!("command is {}", input);
        return RcptTo;
    } else if input.starts_with(DATA) {
        debug!("command is {}", input);
        return Data;
    } else if input.starts_with(QUIT) {
        debug!("command is {}", input);
        return Quit;
    } else if input.starts_with(QUIT) {
        debug!("command is {}", input);
        return Data;
    } else if input.starts_with(AUTH_PLAIN) {
        debug!("command is {}", input);
        return AuthPlain;
    } else if input.starts_with(AUTH_LOGIN) {
        debug!("command is {}", input);
        return AuthLogin;
    } else if input.starts_with(STARTTLS) {
        debug!("command is {}", input);
        return StartTls;
    }

    debug!("command is {} [Unknown]", input);
    return Unknown;
}
