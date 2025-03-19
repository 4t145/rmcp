use std::{collections::VecDeque, marker::PhantomData, task::ready};

use crate::service::{RxJsonRpcMessage, ServiceRole, TxJsonRpcMessage};

use super::IntoTransport;
use futures::{Sink, Stream};
use tower_service::Service as TowerService;
pub enum TransportAdapterTower {}


// impl<R, E, S> IntoTransport<R, E, TransportAdapterTower> for S 
// where S: TowerService<RxJsonRpcMessage<R>, Response = TxJsonRpcMessage<R>, Error = E> + Send + 'static,
//       R: ServiceRole,
//       E: std::error::Error + Send + 'static,
// {
//     fn into_transport(
//         self,
//     ) -> (
//         impl Sink<TxJsonRpcMessage<R>, Error = E> + Send + 'static,
//         impl Stream<Item = RxJsonRpcMessage<R>> + Send + 'static,
//     ) {
//         let (sink, rx) = tokio::sync::mpsc::un();
//         (tokio_util::sync::PollSender::new(sink), tokio_util::io::)
//     }
// }
use std::pin::Pin;


pin_project_lite::pin_project! {
    pub struct TowerServiceSink<R, S: TowerService<R>> {
        #[pin]
        service: S,
        tasks: tokio::task::JoinSet<Result<S::Response, S::Error>>,
        _marker: PhantomData<fn (R)>,
    }
}

impl<Req, S> Sink<Req> for TowerServiceSink<Req, S> 
where S: TowerService<Req> + Unpin,
S::Future : Send + 'static,
S::Response: Send + 'static,
S::Error: Send + 'static,
{
    type Error = S::Error;

    fn poll_ready(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.as_mut().project();
 
        let service = this.service;
        service.get_mut().poll_ready(cx)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Req) -> Result<(), Self::Error> {
        let this = self.as_mut().project();
        let service = this.service;
        let fut = service.get_mut().call(item);
        this.tasks.spawn(fut);
        Ok(())
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.poll_flush(cx)
    }
}


// impl<Req, S> Stream for TowerServiceSink<Req, S> 
// where S: TowerService<Req>
// {
//     type Item = S::Response;

//     fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//         let this = self.project();
//         let tasks = this.tasks;
//         let result = ready!(tasks.poll_join_next(cx));
//         result.map(|join_result|join_result.expect("tokio join error"))

//     }
// }