use crate::messages::*;
use crate::{config::Config, state::State};
use actix::{Actor, AsyncContext, Handler, StreamHandler};
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use log::{error, info};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WebSocketPacket {
    pub id: u8,
    pub data: ChannelMessage,
}

impl From<ChannelMessage> for WebSocketPacket {
    fn from(msg: ChannelMessage) -> Self {
        let id = match msg {
            ChannelMessage::UpdateLyricLine(_) => 0,
            ChannelMessage::UpdateMusicInfo(_) => 1,
            ChannelMessage::UpdateConfig(_) => 2,
        };
        WebSocketPacket { id, data: msg }
    }
}

pub(crate) struct LyricaWebSocketActor {
    rx: broadcast::Receiver<ChannelMessage>,
    config: Arc<RwLock<Config>>,
    state: Arc<RwLock<State>>,
}

impl Actor for LyricaWebSocketActor {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket connection established");

        let mut rx = self.rx.resubscribe();
        let ctx_address = ctx.address();

        let state = self.state.clone();
        let fut = async move {
            // Send current status to client immediately after connection
            let state = state.read().await;
            ctx_address.do_send(ChannelMessage::update_music_info(
                state.title.clone(),
                state.artist.clone(),
            ));
            ctx_address.do_send(ChannelMessage::update_lyric_line(
                0,
                state.text.clone(),
                state.alt.clone(),
            ));
            drop(state);
            while let Ok(msg) = rx.recv().await {
                ctx_address.do_send(msg);
            }
        };
        ctx.spawn(actix::fut::wrap_future(fut));
    }
}

impl actix::Message for ChannelMessage {
    type Result = ();
}

impl Handler<ChannelMessage> for LyricaWebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: ChannelMessage, ctx: &mut Self::Context) {
        // convert message to WebSocketPacket and send to client
        info!("{}", &msg);

        let packet = WebSocketPacket::from(msg);
        if let Ok(text) = serde_json::to_string(&packet) {
            ctx.text(text);
        } else {
            error!("Failed to serialize WebSocketPacket");
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LyricaWebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub(crate) async fn ws_index(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<broadcast::Sender<ChannelMessage>>,
    config: web::Data<RwLock<Config>>,
    state: web::Data<RwLock<State>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        LyricaWebSocketActor {
            rx: tx.subscribe(),
            config: config.into_inner(),
            state: state.into_inner(),
        },
        &req,
        stream,
    )
}
