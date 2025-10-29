//! Entrypoint that delegates BLE work to the `ble` module.

pub mod ble;
pub mod server;
pub mod proto;

use bytes::Bytes;
use tokio::sync::broadcast;

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    //let (to_ble, from_server) = mpsc::channel::<Bytes>(16);
    //let (to_server, from_ble) = broadcast::channel::<Bytes>(16);

    //let to_ble = PeerPair::<Bytes>::new(16);
    //let to_server = PeerPair::<Bytes>::new(16);

    // Broadcast channels 
    let (server_broadcaster, _) = broadcast::channel::<Bytes>(16);
    let (ble_broadacaster, _) = broadcast::channel::<Bytes>(16);

    // The server will get a subscription for each spawned connect task
    // out of the ble broadcaster.
    let server_provider = ble_broadacaster.clone();

    // Subscription to server notifications for BLE.
    let ble_subs = server_broadcaster.subscribe();

    tokio::spawn(async move {
        if let Err(err) = ble::configure(ble_subs, ble_broadacaster).await {
            eprintln!("ble::configure failed: {:?}", err);
        }
    });        // BLE task

    server::run(server_provider, server_broadcaster).await?;       // or spawn as well

    Ok(())
}