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
use tokio::{sync::Mutex, time::timeout};
use std::{net::SocketAddr, ops::ControlFlow, sync::Arc};
use std::time::Duration;
use bytes::Bytes;

use crate::{app_state::{AppState, ClientHandle, ClientsState}, auth::Claims, client::WsClient};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    _: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let clients = state.clients();
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, clients)
    })
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, clients: Arc<ClientsState>) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        let client = clients.insert_client(WsClient::new()).await;
        tracing::info!("New client {}", client.lock().await.id());
        loop {
            match get_message(&mut receiver).await {
                Err(_) => match sender.send(Message::Ping(Bytes::new())).await {
                    Ok(_) => {}
                    Err(_) => {
                        stop_processing(clients, client).await;
                        break;
                    }
                }
                Ok(msg) => match msg {
                    None => {
                        stop_processing(clients, client).await;
                        break;
                    }
                    Some(m) => if process_message(m, who).is_break() {
                        stop_processing(clients, client).await;
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

async fn stop_processing(clients: Arc<ClientsState>, client: ClientHandle) {
    tracing::info!("Close client with id {}", client.lock().await.id());
    clients.del_client(client.lock().await.id().to_string().as_str()).await;
}
