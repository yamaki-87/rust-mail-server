use crate::constants::*;

pub enum Command {
    Helo,
    Ehlo,
    MailFrom,
    RcptTo,
    Data,
    Quit,
    AuthPlain,
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
        return Helo;
    } else if input.starts_with(EHLO) {
        return Ehlo;
    } else if input.starts_with(MAILFROM) {
        return MailFrom;
    } else if input.starts_with(RCPTTO) {
        return RcptTo;
    } else if input.starts_with(DATA) {
        return Data;
    } else if input.starts_with(QUIT) {
        return Quit;
    } else if input.starts_with(QUIT) {
        return Data;
    } else if input.starts_with(AUTH_PLAIN) {
        return AuthPlain;
    }

    return Unknown;
}
