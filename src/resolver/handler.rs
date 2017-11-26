use std::net::SocketAddr;
use std::cell::RefCell;

use futures::future;
use futures::Future;

use tokio_core::reactor::Handle;
use trust_dns::error::*;
use trust_dns::op::message::Message;
use trust_dns::op::Query;
use trust_dns::udp::UdpClientStream;
use trust_dns::tcp::TcpClientStream;
use trust_dns::client::ClientFuture;
use trust_dns::client::BasicClientHandle;
use trust_dns::client::ClientHandle;
use trust_dns_server::server::Request;

use tokio_timer::Timer;
use std::time::Duration;

pub struct SmartResolver {
    fut_client: RefCell<BasicClientHandle>,
    fut_tcp: RefCell<BasicClientHandle>,
}

impl SmartResolver {
    pub fn new(sa: SocketAddr, handle: Handle)-> Result<SmartResolver, ClientError> {

        let (streamfut, streamhand) = UdpClientStream::new(sa, &handle.clone());
        let futclient = ClientFuture::new(streamfut, streamhand, &handle.clone(), None);

        let (streamfut, streamhand) = TcpClientStream::new(sa, &handle.clone());
        let futtcp = ClientFuture::new(streamfut, streamhand, &handle.clone(), None);
        Ok(SmartResolver {
            fut_client: RefCell::new(futclient),
            fut_tcp: RefCell::new(futtcp),
        })
    }

    pub fn handle_future(&self, req: &Request, use_tcp:bool) -> Box<Future<Item=Message, Error=ClientError>> {
        let queries = req.message.queries();
        if queries.len() != 1 {
            panic!("more than 1 queries in a message: {:?}", req.message);
        }
        let q: &Query = &queries[0];
        let name = q.name();
        println!("querying {:?}", name);
        let id = req.message.id();

        let timer = Timer::default();
        let timeout = timer.sleep(Duration::from_secs(5));

        let client = if use_tcp { &self.fut_tcp } else { &self.fut_client };
        let res = client.borrow_mut()
            .query(name.clone(), q.query_class(), q.query_type());
        let m
        = res.and_then(move|result| {
            let mut msg = Message::new();
            msg.set_id(id);
            for ans in result.answers() {
                msg.add_answer(ans.clone());
            }
            timeout.then(|_| future::ok(msg))
        });
        Box::new(m)
    }

}
