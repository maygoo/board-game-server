use std::sync::mpsc::{channel, Sender, Receiver};
use gloo_net::websocket::Message as WsMessage;
use wasm_bindgen_futures::spawn_local;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;

use common::{WAIT, ChannelBuf};
use crate::log;

pub struct Worker {
    pub tx: Sender<ChannelBuf>,
    pub rx: Receiver<ChannelBuf>,
}

impl Worker {
    pub fn new(mut ws: WebSocket) -> Self {
        let (tx_t, rx) = channel::<ChannelBuf>();
        let (tx, rx_t) = channel::<ChannelBuf>();
        
        spawn_local(async move {
            log!("Connected to websocket");

            loop {
                // should equate to a thread::sleep
                gloo_timers::future::sleep(WAIT).await;

                // check for any incoming messages on the websocket
                if let futures::task::Poll::
                    Ready(Some(Ok(WsMessage::Bytes(bytes)))) = futures::poll!(ws.next()) {
                        // forward message through the channel
                        //log!("msg: {bytes:?}");
                        tx_t.send(bytes).unwrap();
                    }

                // check for any incoming messages on the channel
                if let Ok(msg) = rx_t.try_recv() {
                    // forward message through the websocket
                    ws.send(WsMessage::Bytes(msg)).await.unwrap();
                }
            }
        });

        Worker {
            tx,
            rx,
        }
    }
}