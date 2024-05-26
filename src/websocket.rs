use actix::{Actor, AsyncContext, ActorFuture, StreamHandler, Handler};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use tokio::sync::broadcast;

use crate::ChannelMessage;

pub(crate) struct LyricaSocket {
    rx: broadcast::Receiver<ChannelMessage>,
}

impl Actor for LyricaSocket {
    type Context = ws::WebsocketContext<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WebSocket connection established");

        let mut rx = self.rx.resubscribe();
        let ctx_address = ctx.address();

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
            ChannelMessage::UpdateLyricLine(lyric_line) => {
                ctx.text(lyric_line);
            }
            ChannelMessage::UpdateMusicInfo(title, artist) => {
                ctx.text(format!("{} - {}", title, artist));
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
) -> Result<HttpResponse, Error> {
    ws::start(LyricaSocket {
        rx: tx.subscribe(),
    }, &req, stream)
}
