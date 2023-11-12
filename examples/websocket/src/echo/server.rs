use iced::futures;

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use warp::ws::WebSocket;
use warp::Filter;

// Basic WebSocket echo server adapted from:
// https://github.com/seanmonstar/warp/blob/3ff2eaf41eb5ac9321620e5a6434d5b5ec6f313f/examples/websockets_chat.rs
//
// Copyright (c) 2018-2020 Sean McArthur
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
pub async fn run() {
    let routes = warp::path::end()
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| ws.on_upgrade(user_connected));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn user_connected(ws: WebSocket) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (mut tx, mut rx) = mpsc::unbounded();

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx.send(message).await.unwrap_or_else(|e| {
                eprintln!("websocket send error: {e}");
            });
        }
    });

    while let Some(result) = user_ws_rx.next().await {
        let Ok(msg) = result else { break };

        let _ = tx.send(msg).await;
    }
}
