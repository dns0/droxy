use std::net::SocketAddr;

use futures::Future;
use futures::IntoFuture;

use trust_dns::error::*;
use trust_dns::op::message::Message;
use trust_dns::op::Query;
use trust_dns::udp::UdpClientConnection;
use trust_dns::tcp::TcpClientConnection;
use trust_dns::client::{Client, SyncClient};
use trust_dns_server::server::Request;

pub struct SmartResolver {
    conn: SyncClient,
    tcp_client: SyncClient,
}

impl SmartResolver {
    pub fn new(sa: SocketAddr)-> Result<SmartResolver, ClientError> {
        let client = UdpClientConnection::new(sa);
        let client: UdpClientConnection = client?;
        let client =SyncClient::new(client);
        let tcpconn = TcpClientConnection::new(sa)?;
        let tcpclient = SyncClient::new(tcpconn);
        Ok(SmartResolver {
            conn: client,
            tcp_client: tcpclient,
        })
    }

    pub fn handle_future(&self, req: &Request, use_tcp:bool) -> Box<Future<Item=Message, Error=ClientError>> {
        let res = self.handle_request(req, use_tcp);
        Box::new(res.into_future())
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
