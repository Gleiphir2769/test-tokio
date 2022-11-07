use std::sync::RwLock;

use crate::{Connection, Shutdown};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};

#[derive(Debug)]
struct Listener {
    listener: TcpListener,

    notify_shutdown: broadcast::Sender<()>,
}

#[derive(Debug)]
struct Handler {
    shutdown: Shutdown,

    connection: RwLock<Connection>,
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            let mut connection = self.connection.write().unwrap();
            let maybe = tokio::select! {
                res = connection.read() => res?,
                _ = self.shutdown.recv() => return Ok(())
            };

            let res = match maybe {
                Some(v) => v,
                None => return Ok(()),
            };
            self.connection.write().unwrap().write(res);
        }

        Ok(())
    }
}
