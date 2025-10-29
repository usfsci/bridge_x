use tokio::sync::{
    broadcast
};
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Peer<T> {
    tx: broadcast::Sender<T>,
}

impl<T> Peer<T>
where 
    T: Clone + Send + Debug + 'static,
{
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn send(&self, msg: T) {
        let _ = self.tx.send(msg);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.tx.subscribe()
    }
}

#[derive(Debug, Clone)]
pub struct PeerPair<T> {
    pub a: Peer<T>,
    pub b: Peer<T>,
}

impl<T> PeerPair<T>
where
    T: Clone + Send + Debug + 'static,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            a: Peer::new(capacity),
            b: Peer::new(capacity),
        }
    }
}
