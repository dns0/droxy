use std::net::SocketAddr;
use std::cell::RefCell;

use futures::future;
use futures::Future;
use futures::IntoFuture;

use tokio_core::reactor::Handle;
use trust_dns::error::*;
use trust_dns::op::message::Message;
use trust_dns::op::Query;
use trust_dns::udp::UdpClientConnection;
use trust_dns::udp::UdpClientStream;
use trust_dns::tcp::TcpClientConnection;
use trust_dns::client::{Client, SyncClient};
use trust_dns::client::ClientFuture;
use trust_dns::client::BasicClientHandle;
use trust_dns::client::ClientHandle;
use trust_dns_server::server::Request;

use tokio_timer::Timer;
use std::time::Duration;

pub struct SmartResolver {
    conn: SyncClient,
    tcp_client: SyncClient,
    fut_client: RefCell<BasicClientHandle>,
}

impl SmartResolver {
    pub fn new(sa: SocketAddr, handle: Handle)-> Result<SmartResolver, ClientError> {
        let client = UdpClientConnection::new(sa);
        let client: UdpClientConnection = client?;
        let client =SyncClient::new(client);
        let tcpconn = TcpClientConnection::new(sa)?;
        let tcpclient = SyncClient::new(tcpconn);

        let (streamfut, streamhand) = UdpClientStream::new(sa, &handle.clone());
        let futclient = ClientFuture::new(streamfut, streamhand, &handle.clone(), None);
        Ok(SmartResolver {
            conn: client,
            tcp_client: tcpclient,
            fut_client: RefCell::new(futclient),
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

        let res = self.fut_client.borrow_mut()
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

    pub fn handle_request(&self, req: &Request, use_tcp: bool) -> Result<Message, ClientError> {
        let queries = req.message.queries();
        if queries.len() > 1 {
            debug!("more than 1 queries in a message: {:?}", req.message);
        }
        if queries.len() < 1 {
            panic!("empty query");
        }
        let q: &Query = &queries[0];
        let name = q.name();
        println!("querying {:?}", name);

        let client = if use_tcp { &self.tcp_client } else { &self.conn };
        let result: ClientResult<Message> = client.query(
            name, q.query_class(), q.query_type());
        let result: Message = result?;
        let mut msg = Message::new();
        msg.set_id(req.message.id());
        for ans in result.answers() {
            msg.add_answer(ans.clone());
        }
        Ok(msg)
    }
}
