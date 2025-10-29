//! Entrypoint that delegates BLE work to the `ble` module.

pub mod ble;
pub mod server;
pub mod proto;

use bytes::Bytes;
use tokio::sync::{broadcast, mpsc};

use crate::server::peer::PeerPair;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    //let (to_ble, from_server) = mpsc::channel::<Bytes>(16);
    //let (to_server, from_ble) = broadcast::channel::<Bytes>(16);

    let to_ble = Peer::<Bytes>::new(16);
    let to_server = Peer::<Bytes>::new(16);

    tokio::spawn(async move {
        if let Err(err) = ble::configure(from_server, to_server).await {
            eprintln!("ble::configure failed: {:?}", err);
        }
    });        // BLE task

    server::run(to_server, to_ble).await?;       // or spawn as well

    Ok(())
}