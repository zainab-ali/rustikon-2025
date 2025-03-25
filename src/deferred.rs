use futures::channel::oneshot::{channel, Receiver, Sender};

pub struct Deferred {
    recv: Option<Receiver<()>>,
    senders: Vec<Sender<()>>,
}

impl Deferred {
    pub fn new(recv: Receiver<()>) -> Deferred {
        Deferred {
            recv: Some(recv),
            senders: vec![],
        }
    }

    pub async fn wait(&mut self) {
        if let Some(recv) = self.recv.take() {
            recv.await.unwrap();
        }
        while let Some(sender) = self.senders.pop() {
            sender.send(()).unwrap();
        }
    }

    pub async fn get(&mut self) -> () {
        if self.recv.is_some() {
            let (sender, recv) = channel::<()>();
            self.senders.push(sender);
            recv.await.unwrap()
        }
    }
}
