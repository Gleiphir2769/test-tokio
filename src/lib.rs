pub mod server;

mod shutdown;
use shutdown::Shutdown;

mod connection;
pub use connection::Connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;