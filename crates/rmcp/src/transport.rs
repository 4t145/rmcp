use futures::{Sink, Stream};
#[cfg(feature = "transport-child-process")]
pub mod child_process;

#[cfg(feature = "transport-io")]
pub mod io;

#[cfg(feature = "transport-sse")]
pub mod sse;

pin_project_lite::pin_project! {
    pub struct Transport<Tx, Rx> {
        #[pin]
        rx: Rx,
        #[pin]
        tx: Tx,
    }
}

impl<Tx, Rx> Transport<Tx, Rx> {
    pub fn new(tx: Tx, rx: Rx) -> Self {
        Self { tx, rx }
    }
}
impl<Tx, Rx> Stream for Transport<Tx, Rx>
where
    Rx: Stream,
{
    type Item = Rx::Item;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.rx.poll_next(cx)
    }
}

impl<Tx, Rx, T> Sink<T> for Transport<Tx, Rx>
where
    Tx: Sink<T>,
{
    type Error = Tx::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.tx.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = self.project();
        this.tx.start_send(item)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.tx.poll_flush(cx)
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.tx.poll_close(cx)
    }
}
