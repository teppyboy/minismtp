pub static EHLO_TLS_AVAILABLE: &str = "250-%DOMAIN%\n250 STARTTLS\n";
pub static EHLO_TLS_UNAVAILABLE: &str = "250 %DOMAIN%\n";
pub static OK: &str = "250 OK\n";
pub static READY_FOR_TLS: &str = "220 Ready to start TLS\n";
pub static TLS_NOT_AVAILABLE: &str = "502 TLS not available\n";