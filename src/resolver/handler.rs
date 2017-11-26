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
    server: SocketAddr,
    handle: Handle,
}

impl SmartResolver {
    pub fn new(sa: SocketAddr, handle: Handle)-> Result<SmartResolver, ClientError> {

        let (streamfut, streamhand) = UdpClientStream::new(sa, &handle.clone());
        let futclient = ClientFuture::new(streamfut, streamhand, &handle.clone(), None);

        Ok(SmartResolver {
            fut_client: RefCell::new(futclient),
            server: sa,
            handle: handle,
        })
    }

    fn get_tcp_client(&self)-> BasicClientHandle {
        let (streamfut, streamhand) = TcpClientStream::new(self.server, &self.handle.clone());
        let futtcp = ClientFuture::new(streamfut, streamhand, &self.handle.clone(), None);
        futtcp
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

        let res = if use_tcp {
            let mut c = self.get_tcp_client();
            c.query(name.clone(), q.query_class(), q.query_type())
        } else {
            self.fut_client.borrow_mut().query(name.clone(), q.query_class(), q.query_type())
        };
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
