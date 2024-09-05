// Responses as per RFC 5321
// The following are the responses that the server can send to the client as per RFC 5321:
pub static EHLO_TLS_AVAILABLE: &[u8] = b"250-%DOMAIN%\r\n250 STARTTLS\r\n";
pub static EHLO_TLS_UNAVAILABLE: &[u8] = b"250 %DOMAIN%\r\n";
pub static OK: &[u8] = b"250 OK\r\n";
pub static READY_FOR_TLS: &[u8] = b"220 Ready to start TLS\r\n";
pub static TLS_NOT_AVAILABLE: &[u8] = b"502 TLS not available\r\n";
pub static SEND_DATA: &[u8] = b"354 Start mail input; end with <CRLF>.<CRLF>\r\n";
pub static QUIT: &[u8] = b"221 Bye\r\n";
