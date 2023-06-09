use crate::files;
use axum::{
    body::{self, Bytes, HttpBody},
    response::{IntoResponse, Response},
    BoxError, Error,
};
use futures_util::{ready, stream::TryStream};
use http::HeaderMap;
use pin_project_lite::pin_project;
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use sync_wrapper::SyncWrapper;

pub enum StreamBodySenderMessage {
    Chunk(Bytes),
    Finished,
}

pin_project! {
    /// A `StreamBody` clone that sends the stream contents across an mpsc channel
    #[must_use]
    pub struct StreamBodySender<S> {
        #[pin]
        stream: SyncWrapper<S>,

        send_channel: Option<mpsc::Sender<StreamBodySenderMessage>>,
    }
}

impl<S> StreamBodySender<S> {
    /// Create a new `StreamBody` from a [`Stream`].
    ///
    /// [`Stream`]: futures_util::stream::Stream
    pub fn new(stream: S, send_channel: Option<mpsc::Sender<StreamBodySenderMessage>>) -> Self
    where
        S: TryStream + Send + 'static,
        S::Ok: Into<Bytes>,
        S::Error: Into<BoxError>,
    {
        Self {
            stream: SyncWrapper::new(stream),
            send_channel,
        }
    }
}

impl<S> IntoResponse for StreamBodySender<S>
where
    S: TryStream + Send + 'static,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    fn into_response(self) -> Response {
        Response::new(body::boxed(self))
    }
}

impl<S> fmt::Debug for StreamBodySender<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("StreamBodySender").finish()
    }
}

impl<S> HttpBody for StreamBodySender<S>
where
    S: TryStream,
    S::Ok: Into<Bytes>,
    S::Error: Into<BoxError>,
{
    type Data = Bytes;
    type Error = Error;

    fn poll_data(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        let send_channel = self.send_channel.clone();

        let stream = self.project().stream.get_pin_mut();
        match ready!(stream.try_poll_next(cx)) {
            Some(Ok(chunk)) => {
                let mut chunk_data = chunk.into();

                // Delete carriage return in text responses
                if let Ok(text) = std::str::from_utf8(&chunk_data) {
                    let new_text = text.replace('\r', "");
                    chunk_data = Bytes::from(new_text);
                }

                if let Some(send_channel) = send_channel {
                    send_channel
                        .send(StreamBodySenderMessage::Chunk(chunk_data.clone()))
                        .unwrap();
                }

                Poll::Ready(Some(Ok(chunk_data)))
            }
            Some(Err(err)) => {
                if let Some(send_channel) = send_channel {
                    send_channel
                        .send(StreamBodySenderMessage::Finished)
                        .unwrap();
                }

                Poll::Ready(Some(Err(Error::new(err))))
            }
            None => {
                if let Some(send_channel) = send_channel {
                    send_channel
                        .send(StreamBodySenderMessage::Finished)
                        .unwrap();
                }

                Poll::Ready(None)
            }
        }
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

/// Write the data of a `StreamBodySender` to a file (`dest`)
pub fn write_stream_to_file(dest: &PathBuf, receiver: Receiver<StreamBodySenderMessage>) {
    let mut result = Vec::new();

    let mut succeeded = false;
    while let Ok(message) = receiver.recv() {
        match message {
            StreamBodySenderMessage::Chunk(chunk) => {
                for byte in chunk {
                    result.push(byte);
                }
            }
            StreamBodySenderMessage::Finished => {
                succeeded = true;
                break;
            }
        }
    }

    if succeeded {
        files::write_file(dest, result).unwrap();
    }
}
