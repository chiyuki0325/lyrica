use actix::{Actor, AsyncContext, StreamHandler, Handler};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use tokio::sync::broadcast;
use serde::Serialize;

use crate::ChannelMessage;
use crate::config::SharedConfig;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WebSocketPacket {
    pub id: u8,
    pub data: WebSocketPacketData,
}

#[derive(Debug, Clone, Serialize)]
#[allow(non_camel_case_types)]
pub(crate) enum WebSocketPacketData {
    lyric_line(UpdateLyricLinePacket),
    music_info(UpdateMusicInfoPacket),
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateLyricLinePacket {
    lyric: String,
    time: u128,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UpdateMusicInfoPacket {
    title: String,
    artist: String,
}

pub(crate) struct LyricaSocket {
    rx: broadcast::Receiver<ChannelMessage>,
    config: SharedConfig,
}

impl Actor for LyricaSocket {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket connection established");

        let mut rx = self.rx.resubscribe();
        let ctx_address = ctx.address();


        ctx_address.do_send(ChannelMessage::UpdateMusicInfo(
            String::new(),
            String::new(),
        ));

        let fut = async move {
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

impl Handler<ChannelMessage> for LyricaSocket {
    type Result = ();

    fn handle(&mut self, msg: ChannelMessage, ctx: &mut Self::Context) {
        match msg {
            ChannelMessage::UpdateLyricLine(time, lyric) => {
                if self.config.read().unwrap().verbose {
                    println!("[{time}] {lyric}");
                }

                let packet = WebSocketPacket {
                    id: 1,
                    data: WebSocketPacketData::lyric_line(UpdateLyricLinePacket {
                        lyric, time,
                    }),
                };
                ctx.text(serde_json::to_string(&packet).unwrap());
            }
            ChannelMessage::UpdateMusicInfo(title, artist) => {
                if self.config.read().unwrap().verbose {
                    println!("[{title} - {artist}]");
                }

                let packet = WebSocketPacket {
                    id: 0,
                    data: WebSocketPacketData::music_info(UpdateMusicInfoPacket {
                        title, artist,
                    }),
                };
                ctx.text(serde_json::to_string(&packet).unwrap());
            }
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LyricaSocket {
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
    config: web::Data<SharedConfig>,
) -> Result<HttpResponse, Error> {
    ws::start(LyricaSocket {
        rx: tx.subscribe(),
        config: config.get_ref().clone(),
    }, &req, stream)
}
