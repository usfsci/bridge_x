use futures::{
    SinkExt,
    StreamExt
};
use tokio::task;
use tokio::net::{UnixListener, UnixStream};
use tokio_util::codec::{Framed};
//use tokio_stream; 
use tokio::sync::{
    broadcast
};
use bytes::{Bytes, BytesMut, BufMut};
use std::fs;
use crate::server::mcodec::{TwoByteLenSkipReserved, MAX_FRAME_SIZE};
//use crate::server::peer::{Peer, PeerPair};
use crate::proto::msg::{Message, Response, decode_message};


const SOCKET_FILE: &str = "/tmp/gateway.sock";


pub async fn run(provider: broadcast::Sender::<Bytes>, broadcaster: broadcast::Sender::<Bytes>) -> std::io::Result<()> {
    let _ = fs::remove_file(SOCKET_FILE);

    let listener = UnixListener::bind(SOCKET_FILE)?;

    loop {
        let (stream, _) = listener.accept().await?;

        let transmitter = broadcaster.clone();
        let subs = provider.subscribe();
        
        tokio::spawn(async move {
            handle_connection(stream, subs, transmitter).await;
        });
    }
}

async fn handle_connection(stream: UnixStream, mut subs: broadcast::Receiver<Bytes>, transmitter: broadcast::Sender<Bytes>) {
    // Use FramedRead (read-only) with our codec
    //let reader = FramedRead::new(stream, TwoByteLenSkipReserved::new(MAX_FRAME_SIZE));
    // use a single Framed (Stream + Sink) to read frames and write responses
    //let mut reader_writer = Framed::new(stream, TwoByteLenSkipReserved::new(MAX_FRAME_SIZE));
    let codec = TwoByteLenSkipReserved::new(MAX_FRAME_SIZE);
    let framed = Framed::new(stream, codec);

    // split into sink (writer) and stream (reader)
    let (mut sink, mut source) = framed.split();

    //let (tx, rx) = mpsc::channel(16);

    let _reader_task = task::spawn(async move {
        while let Some(frame_res) = source.next().await {
            match frame_res {
                Ok(bytes_payload) => {
                    // Rebuild the frame and forward to BLE
                    let mut forward_frame = BytesMut::new();
                    forward_frame.put_slice(&[0xff, 0xff]);
                    let len_field = bytes_payload.len() as u16;
                    forward_frame.put_slice(&len_field.to_be_bytes());
                    forward_frame.put(bytes_payload.clone());
                    if let Err(e) = transmitter.send(forward_frame.freeze()) {
                        println!("transmitter err: {}", e);
                    }

                    let _r = match decode_message(bytes_payload) {
                        Ok(m) => {
                            match m {
                                Message::Request(req) => {
                                    println!("req=\n{}", req);
                                    /*if let Err(e) = transmitter.send(Bytes::from(req.encode())) {
                                        println!("transmitter err: {}", e);
                                    }*/

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

                                    if let Err(e) = sink.send(payload).await {
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
                    //break; // or continue depending on desired policy
                }
            }
        }
    });

    let _writer_task = task::spawn(async move {
        loop {
            let ble_msg = subs.recv().await;
            match ble_msg {
                Ok(m) => {
                    println!("recv ble msg: {:?}", m);
                    /*let response = Response::new(
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

                    if let Err(e) = sink.send(payload).await {
                        eprintln!("could not send response: {}", e);
                        return;
                    }*/
                }
                Err(e) => {
                    eprintln!("subs recv err: {}", e);
                }
            }
        }
    });

    /*loop {
        tokio::select! {
            Some(frame_res) = source.next() => {
                match frame_res {
                    Ok(bytes_payload) => {
                        let _r = match decode_message(bytes_payload) {
                            Ok(m) => {
                                match m {
                                    Message::Request(req) => {
                                        println!("req=\n{}", req);
                                        if let Err(e) = transmitter.send(Bytes::from("sent a request to channel")) {
                                            println!("transmitter err: {}", e);
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
                                        if let Err(e) = sink.send(payload).await {
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
                        //break; // or continue depending on desired policy
                    }
                }
            }

            ble_msg = subs.recv() => {
                match ble_msg {
                    Ok(m) => println!("recv ble msg: {:?}", m),
                    Err(e) => eprintln!("subs recv err: {}", e)
                }
            }
        }
    }*/


    /*while let Some(frame_res) = source.next().await {
        match frame_res {
            Ok(bytes_payload) => {
                let _r = match decode_message(bytes_payload) {
                    Ok(m) => {
                        match m {
                            Message::Request(req) => {
                                println!("req=\n{}", req);
                                if let Err(e) = transmitter.send(Bytes::from("sent a request to channel")) {
                                    println!("transmitter err: {}", e);
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
                                if let Err(e) = sink.send(payload).await {
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
    }*/

    println!("connection closed");
}


/*async fn handle_connection(stream: UnixStream, subs: broadcast::Receiver<Bytes>, transmitter: broadcast::Sender<Bytes>) {
    // Use FramedRead (read-only) with our codec
    //let reader = FramedRead::new(stream, TwoByteLenSkipReserved::new(MAX_FRAME_SIZE));
    // use a single Framed (Stream + Sink) to read frames and write responses
    let mut reader_writer = Framed::new(stream, TwoByteLenSkipReserved::new(MAX_FRAME_SIZE));
    
    while let Some(frame_res) = reader_writer.next().await {
        match frame_res {
            Ok(bytes_payload) => {
                let _r = match decode_message(bytes_payload) {
                    Ok(m) => {
                        match m {
                            Message::Request(req) => {
                                println!("req=\n{}", req);
                                if let Err(e) = transmitter.send(Bytes::from("sent a request to channel")) {
                                    println!("transmitter err: {}", e);
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
