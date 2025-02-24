pub const HELO: &str = "HELO";
pub const EHLO: &str = "EHLO";
pub const MAILFROM: &str = "MAIL FROM:";
pub const RCPTTO: &str = "RCPT TO:";
pub const DATA: &str = "DATA";
pub const QUIT: &str = "QUIT";
pub const STARTTLS: &str = "STARTTLS";
pub const AUTH_PLAIN: &str = "AUTH PLAIN";
pub const TEXT_PLAIN: &str = "text/plain";
pub const TEXT_HTML: &str = "text/html";

// Message byte
pub const AUTH_REQUIRED_MESSAGE_BYTES: &[u8] = b"530 Authentication required\r\n";
pub const OK_MESSAGE_BYTES: &[u8] = b"250 Ok\r\n";
pub const STARTTLS_MESSAGE_BYTES: &[u8] = b"220 Ready to start TLS\r\n";
pub const STARTTLS_NO_SUPPORTED_MESSAGE_BYTES: &[u8] = b"500 STARTTLS not supported\r\n";
