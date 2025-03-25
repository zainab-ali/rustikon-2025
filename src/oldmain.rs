#![allow(dead_code)]
mod daemon;
mod unit;
mod command;
mod newunit;
mod orchestrator;
use std::io::prelude::*;
use std::sync::Arc;
use std::time::Duration;
mod deferred;

use deferred::Deferred;
use futures::SinkExt;
use futures::channel::mpsc;
use futures::future::BoxFuture;
use futures::StreamExt;
use futures::future::select;
use std::io::Error;
use std::io::ErrorKind;
use std::process::Command;
use std::collections::hash_map::{Entry, HashMap};

use async_std::{
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    task,
};

use async_std::{io::BufReader, net::TcpStream};

// Q: How is the size of a dyn trait calculated?
// The error is send and sync, meaning it is normal in its ability to be sent between (threads?).
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// Q: what is impl vs dyn?
async fn accept_loop(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();
    let (broker, receiver) = mpsc::unbounded::<Event>();
    let broker_handle = task::spawn(broker_loop(receiver));
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        // Q: Why are we spawning this?
        let handle = spawn_and_log_error(connection_loop(broker.clone(), stream));
    }
    Ok(())
}

async fn connection_loop(mut broker: Sender<Event>, stream: TcpStream) -> Result<()> {
    let stream = Arc::new(stream);
    // Q: How do we use Arc?
    let reader = BufReader::new(&*stream);
    let mut lines = reader.lines();

    let name = match lines.next().await {
        None => Err("disconnected")?,
        Some(line) => line?,
    };
    // Q: Is it safe to rely on drop semantics?
    let (_shutdown_sender, shutdown_receiver) = mpsc::unbounded::<Void>();
    // Q: Does unwrap imply that there is a different error channel?
    broker.send(Event::NewPeer { name: name.clone(), stream: Arc::clone(&stream), shutdown: shutdown_receiver }).await.unwrap();
    while let Some(line) = lines.next().await {
        let line = line?;
        let (dest, msg) = match line.find(":") {
            None => continue,
            Some(idx) => (&line[..idx], &line[idx..]),
        };
        let dest = dest
            .split(",")
            .map(|name| name.to_string())
            .collect::<Vec<_>>();
        let msg = msg.to_string();
	broker.send(Event::Message { from: name.clone(), to: dest, msg }).await.unwrap();
    }
    Ok(())
}
fn spawn_and_log_error<F>(fut: F) -> task::JoinHandle<()>
// Q: You can only spawn static futures? Why doesn't the join handle share the lifetime of the future?
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    // Q: We don't need to move the future?
    let err_fut = async {
        match fut.await {
            Err(err) => eprintln!("{}", err),
            _ => (),
        }
    };
    task::spawn(err_fut)
}

type Sender<T> = mpsc::UnboundedSender<T>;
type Receiver<T> = mpsc::UnboundedReceiver<T>;
use futures::{select, FutureExt};
async fn connection_writer_loop(
    messages: &mut Receiver<String>,
    // Q: Why would we need to arc it?
    stream: Arc<TcpStream>,
    shutdown: Receiver<Void>,
) -> Result<()> {
    // Q: What does &* mean?
    let mut stream = &*stream;
    // Q: What is fusing?
    let mut messages = messages.fuse();
    let mut shutdown = shutdown.fuse();
    loop {
	// Q: What is the syntax of select! ?
	select! {
	    msg = messages.next().fuse() => match msg {
		Some(msg) => stream.write_all(msg.as_bytes()).await?,
		None => break
	    },
	    void = shutdown.next().fuse() => match void {
		Some(void) => match void {}, // Unreachable
		None => break,
	    }
	}
    }
    Ok(())
}
enum Void {}
enum Event {
    NewPeer {
	name: String,
	stream: Arc<TcpStream>,
	shutdown: Receiver<Void>,
    },
    Message {
	from: String,
	to: Vec<String>,
	msg: String
    }
}

async fn broker_loop(mut events: Receiver<Event>) -> Result<()> {
    let mut peers: HashMap<String, Sender<String>> = HashMap::new();
    // Why do we need to "reap" the receivers?
    let (disconnect_sender, mut disconnect_receiver) = mpsc::unbounded::<(String, Receiver<String>)>();
    loop {
	let event = select! {
	    event = events.next().fuse() => match event {
		None => break,
		Some(event) => event,
	    },
	    // Q: What exactly is this doing?
	    disconnect = disconnect_receiver.next().fuse() => {
		// Q: What do we do with the pending messages?
		let (name, _pending_messages) = disconnect.unwrap();
		assert!(peers.remove(&name).is_some());
		continue;
	    }
	};
	match event {
	    Event::Message { from, to, msg} => {
		for addr in to {
		    if let Some(peer) = peers.get_mut(&addr) {
			let msg = format!("from {}: {}\n", from, msg);
			peer.send(msg).await?
		    }
		}
	    },
	    Event::NewPeer { name, stream, shutdown } => {
		match peers.entry(name.clone()) {
		    Entry::Occupied(..) => (),
		    Entry::Vacant(entry) => {
			let (sender, mut receiver) = mpsc::unbounded::<String>();
			entry.insert(sender);
			let mut disconnect_sender = disconnect_sender.clone();
			// Q: What happens to this join handle?
			// Q: Could we instead spawn the receiver from the connection loop actor?
			let handle = spawn_and_log_error(
			    async move {
				let res = connection_writer_loop(&mut receiver, stream, shutdown).await;
				disconnect_sender.send((name, receiver)).await.unwrap();
				res
			    }
			);
		    }
		}
	    }
	}
    }
    drop(peers);
    drop(disconnect_sender);
    while let Some((_name, _pending_messages)) = disconnect_receiver.next().await {
    }
    Ok(())
}
fn main() -> Result<()> {
    let fut = accept_loop("127.0.0.1:8000");
    task::block_on(fut)
}
