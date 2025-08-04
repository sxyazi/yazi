// --- BufRead
pub trait BufRead: tokio::io::AsyncRead + Send {}

impl<T: tokio::io::AsyncRead + Send> BufRead for T {}

// --- BufReadSync
pub trait BufReadSync: std::io::BufRead + std::io::Seek + Send {}

impl<T: std::io::BufRead + std::io::Seek + Send> BufReadSync for T {}
