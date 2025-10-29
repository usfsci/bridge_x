use futures::SinkExt;
//use futures::StreamExt;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed};
use tokio_stream::StreamExt; 
use tokio::sync::{
    mpsc::{Sender, Receiver},
    broadcast
};
//use tokio::task;
use bytes::Bytes;
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::server::mcodec::{TwoByteLenSkipReserved, MAX_FRAME_SIZE};
use crate::server::peer::{Peer, PeerPair};
use crate::proto::msg::{Message, Response, decode_message};


const SOCKET_FILE: &str = "/tmp/gateway.sock";


pub async fn run(rx: PeerPair<Bytes>, tx: PeerPair<Bytes>) -> std::io::Result<()> {
    let _ = fs::remove_file(SOCKET_FILE);

    let listener = UnixListener::bind(SOCKET_FILE)?;

    loop {
        let (stream, _) = listener.accept().await?;
        // spawn a task that handles this connection
        tokio::spawn(handle_connection(stream, rx, tx.clone()));
    }
}

/*async fn handle_connection(stream: UnixStream, tx: Sender<Bytes>) {
    let codec = TwoByteLenSkipReserved::new(MAX_FRAME_SIZE);
    let framed = Framed::new(stream, codec);
    let (mut sink, mut stream) = framed.split();


    while let Some(frame_res) = reader_writer.next().await {
        match frame_res {
            Ok(bytes_payload) => {
                let _r = match decode_message(bytes_payload) {
                    Ok(m) => {
                        match m {
                            Message::Request(req) => {
                                println!("req=\n{}", req);
                                if let Err(_closed) = tx.send(Bytes::from("sent a request to channel")).await {
                                    eprintln!("ble receiver closed");
                                }
                                let response = Response::new(
                                    req.protocol,
                                    req.version,
                                    req.id,
                                    200,
                                    "OK".to_string(),
                                    None
                                );

                                let s = response.encode();
                                println!("will respond: {}", s);

                                let payload = Bytes::from(s);
                                if let Err(e) = reader_writer.send(payload).await {
                                    eprintln!("could not send response: {}", e);
                                    return;
                                }
                            }
                            Message::Response(resp) => println!("resp=\n{}", resp)
                        }

                    },
                    Err(e) => {
                        eprintln!("could not decode message {}", e);
                    }
                };
            }
            Err(e) => {
                eprintln!("frame read error: {}", e);
                break; // or continue depending on desired policy
            }
        }
    }

    println!("connection closed");
}*/

async fn handle_connection(stream: UnixStream, rx: PeerPair<Bytes>, tx: PeerPair<Bytes>) {
    // Use FramedRead (read-only) with our codec
    //let reader = FramedRead::new(stream, TwoByteLenSkipReserved::new(MAX_FRAME_SIZE));
    // use a single Framed (Stream + Sink) to read frames and write responses
    let mut reader_writer = Framed::new(stream, TwoByteLenSkipReserved::new(MAX_FRAME_SIZE));
    let subs = rx.b.subscribe();

    while let Some(frame_res) = reader_writer.next().await {
        match frame_res {
            Ok(bytes_payload) => {
                let _r = match decode_message(bytes_payload) {
                    Ok(m) => {
                        match m {
                            Message::Request(req) => {
                                println!("req=\n{}", req);
                                if let Err(_closed) = tx.send(Bytes::from("sent a request to channel")).await {
                                    eprintln!("ble receiver closed");
                                }

                                let response = Response::new(
                                    req.protocol,
                                    req.version,
                                    req.id,
                                    200,
                                    "OK".to_string(),
                                    None
                                );

                                let s = response.encode();
                                println!("will respond: {}", s);

                                let payload = Bytes::from(s);
                                if let Err(e) = reader_writer.send(payload).await {
                                    eprintln!("could not send response: {}", e);
                                    return;
                                }

                                tx.a.send(payload);
                            }

                            Message::Response(resp) => println!("resp=\n{}", resp)
                        }

                    },
                    Err(e) => {
                        eprintln!("could not decode message {}", e);
                    }
                };
            }
            Err(e) => {
                eprintln!("frame read error: {}", e);
                break; // or continue depending on desired policy
            }
        }
    }

    println!("connection closed");
}
