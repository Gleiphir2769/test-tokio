use std::future::Future;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use crate::{Connection, Shutdown};
use tracing::{debug, error, info, instrument};
use tokio::time::{self, Duration};
use std::str;

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

impl Listener {
    async fn run(&mut self) -> crate::Result<()> {
        info!("accepting inbound connections");
        loop {
            let socket = self.accept().await?;
            
            let mut handler = Handler {
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                connection: Connection::new(socket)
            };
            
            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
            });
        }
    }

    async fn accept(&mut self) -> crate::Result<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err.into());
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
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
            
            let speak = str::from_utf8(&res).unwrap();
            
            println!("received: {}", speak);

            tokio::select! {
                res = self.connection.write(&res) => res?,
                _ = self.shutdown.recv() => {
                    return Ok(())
                }
            };
        }

        Ok(())
    }
}

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);

    // Initialize the listener state
    let mut server = Listener {
        listener,
        notify_shutdown,
    };

    tokio::select! {
        res = server.run() => {
            // If an error is received here, accepting connections from the TCP
            // listener failed multiple times and the server is giving up and
            // shutting down.
            //
            // Errors encountered when handling individual connections do not
            // bubble up to this point.
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            // The shutdown signal has been received.
            info!("shutting down");
        }
    }
}