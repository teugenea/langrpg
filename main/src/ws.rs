use axum::{
    Error,
    Extension,
    extract::{
        ConnectInfo, WebSocketUpgrade,
        ws::{Message, WebSocket},
        State,
    },
    response::{IntoResponse},
};
use axum_extra::TypedHeader;
use futures::stream::StreamExt;
use futures_util::SinkExt;
use futures_util::stream::SplitStream;
use tokio::time::timeout;
use std::{net::SocketAddr, ops::ControlFlow};
use std::time::Duration;
use bytes::Bytes;

use crate::{app_state::AppState, auth::Claims};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    _: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    //tracing::debug!("{}", claims);
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr)
    })
}

async fn handle_socket(socket: WebSocket, who: SocketAddr) {

    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        loop {
            match get_message(&mut receiver).await {
                Err(_) => match sender.send(Message::Ping(Bytes::new())).await {
                    Ok(_) => {}
                    Err(_) => {
                        break;
                    }
                }
                Ok(msg) => match msg {
                    None => break,
                    Some(m) => if process_message(m, who).is_break() {
                        break;
                    }
                }
            }
        }
    });
}

async fn get_message(receiver: &mut SplitStream<WebSocket>) -> Result<Option<Message>, Error> {
    match timeout(Duration::from_millis(5000), receiver.next()).await {
        Ok(t) => match t {
            Some(m) => {
                return match m {
                    Ok(msg) => Ok(Some(msg)),
                    Err(e) => Err(e)
                };
            }
            None => Ok(None)
        }
        Err(_) => Err(Error::new("Timeout"))
    }
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {} sent str: {:?}", who, t);
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {} somehow sent close message without CloseFrame", who);
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {} sent pong with {:?}", who, v);
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {} sent ping with {:?}", who, v);
        }
    }
    ControlFlow::Continue(())
}