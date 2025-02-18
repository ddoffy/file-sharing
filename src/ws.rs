use actix_web::{rt, Error, web, HttpRequest, HttpResponse, Responder};
use actix_ws::AggregatedMessage;
use actix_ws::Message;
use futures_util::StreamExt as _;
use tokio::sync::broadcast;
use tokio::select;

pub async fn ws_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let (res, mut session, stream) = actix_ws::handle(&req, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1 MiB
        .max_continuation_size(2_usize.pow(20));

    rt::spawn(async move {
        // receive messages from websocket
        while let Some(msg) = stream.next().await {
            match msg {
                Ok(AggregatedMessage::Text(text)) => {
                    // echo text message
                    session.text(text).await.unwrap();
                }
                Ok(AggregatedMessage::Binary(bin)) => {
                    // echo binary message
                    session.binary(bin).await.unwrap();
                }
                Ok(AggregatedMessage::Ping(ping)) => {
                    // send pong
                    session.pong(&ping).await.unwrap();
                }
                _ => {},
            }
        }
    });

    Ok(res)
}

async fn broadcast_ws(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    mut rx: broadcast::Receiver<web::Bytes>
) {
    log::info!("connected");

    let reason = loop { 
        select! {
            broadcast_msg = rx.recv() => {
                let msg = match broadcast_msg {
                    Ok(msg) => msg,
                    Err(broadcast::error::RecvError::Closed) => break None,
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                };
                
                let res = match std::str::from_utf8(&msg) {
                    Ok(val) => session.text(val).await,
                    Err(_) => session.binary(msg).await,
                };

                if let Err(err) = res {
                    log::error!("{err}");
                    break None
                }
            }

            msg = msg_stream.next() => {
                let msg = match msg {
                    Some(Ok(msg)) => msg,
                    Some(Err(err)) => {
                        log::error!("{err}");
                        break None;
                    }
                    None => break None,
                };

                match msg {
                    Message::Text(text) => {
                        session.text(text).await.unwrap();
                    }
                    Message::Binary(bin) => {
                        session.binary(bin).await.unwrap();
                    }
                    Message::Close(reason) => {
                        break reason;
                    }
                    Message::Ping(bytes) => {
                        let _ = session.pong(&bytes).await;
                    }
                    Message::Pong(_) => {}
                    Message::Continuation(_) => {
                        log::warn!("Continuation frames are not supported");
                    }
                    Message::Nop => {}
                };
            }
        }

    };

    let _ = session.close(reason).await;

    log::info!("disconnected");
}

pub async fn send_to_broadcast_ws(
    body: web::Bytes,
    tx: web::Data<broadcast::Sender<web::Bytes>>,
) -> Result<impl Responder, Error> {
    tx.send(body).map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent())
}

pub async fn handshake_and_start_broadcast_ws(
    req: HttpRequest,
    stream: web::Payload,
    tx: web::Data<broadcast::Sender<web::Bytes>>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    rt::spawn(broadcast_ws(session, msg_stream, tx.subscribe()));

    Ok(res)
}
