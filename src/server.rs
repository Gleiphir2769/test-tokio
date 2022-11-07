use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use crate::{Connection, Shutdown};

#[derive(Debug)]
struct Listener {
    listener: TcpListener,

    notify_shutdown: broadcast::Sender<()>
}

#[derive(Debug)]
struct Handler {
    shutdown: Shutdown,

    connection: Connection
}

impl Handler {
    async fn run(&mut self) -> crate::Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe = tokio::select! {
                res = self.connection.read() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(())
                }
            };

            let res = match maybe {
                Some(v) => v,
                None => return Ok(()),
            };
            self.connection.write(res);
        }

        Ok(())
    }
}