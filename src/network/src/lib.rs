mod connection;

pub use crate::connection::{Connection, RawPacket};
use raknet_v6::RaknetListener;
use std::net::SocketAddr;
use std::sync::{Arc, LazyLock};
use tracing::info;

const SERVER_PORT: u16 = 16261;
const SERVER_ADDRESS: &str = "127.0.0.1";

pub static CONNECTIONS: LazyLock<dashmap::DashMap<SocketAddr, Arc<Connection>>> =
    LazyLock::new(dashmap::DashMap::new);

pub fn start_net_server() {
    info!("Starting network server...");
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        start_raknet_server().await;
    });
}

async fn start_raknet_server() {
    let addr = SocketAddr::new(SERVER_ADDRESS.parse().unwrap(), SERVER_PORT);
    info!("Listening on: {}", addr);
    let mut server = RaknetListener::bind(&addr).await.unwrap();
    server.listen().await;

    loop {
        let socket = server.accept().await.unwrap();
        info!("New connection from: {}", socket.peer_addr().unwrap());
        let addr = socket.peer_addr().unwrap();
        let (connection, outbound_messages) = Connection::new(socket);
        let conn = Arc::new(connection);
        CONNECTIONS.insert(addr, conn.clone());
        tokio::spawn(async move {
            conn.clone().start(outbound_messages).await;
            CONNECTIONS.remove_if(&addr, |_, current| Arc::ptr_eq(current, &conn));
        });
    }
}
