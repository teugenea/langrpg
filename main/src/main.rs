use std::{net::SocketAddr, ops::ControlFlow};
use std::time::{Duration, Instant};

use axum::{Error, extract::{
    ConnectInfo,
    WebSocketUpgrade, ws::{Message, WebSocket},
}, headers, response::IntoResponse, Router, routing::get, TypedHeader};
use futures::stream::StreamExt;
use futures_util::SinkExt;
use futures_util::stream::SplitStream;
use tokio::time::timeout;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::client::WsClient;

pub mod client;

// https://github.com/tokio-rs/axum/blob/v0.6.x/examples/websockets/src/main.rs

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        // .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(ws_handler))
        // logging so we can see whats going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // run it with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr)
    })
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    tokio::spawn(async move {
        let mut client = WsClient::new();
        loop {
            match get_message(&mut receiver).await {
                Err(_) => match sender.send(Message::Ping(vec![1, 2, 3])).await {
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
        Err(e) => Err(Error::new("Timeout"))
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