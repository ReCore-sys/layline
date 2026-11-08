use bytes::{Buf, BufMut, Bytes, BytesMut};
use layline_codec::{NetEncode, Packet};
use raknet_v6::{RaknetSocket, Reliability};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

const PACKET_MARKER: u8 = 0x86;

/// An inbound packet whose payload hasn't been decoded yet.
pub struct RawPacket {
    pub id: i16,
    pub bytes: Bytes,
}

impl RawPacket {
    fn read(mut bytes: Bytes) -> Option<Self> {
        if bytes.len() < 3 {
            warn!("Incoming packet has a truncated header");
            return None;
        }
        let marker = bytes.get_u8();
        if marker != PACKET_MARKER {
            warn!("Got non-0x86 marker for incoming packet: 0x{:02X}", marker);
            return None;
        }
        let id = bytes.get_i16();
        Some(Self { id, bytes })
    }
}

pub struct Connection {
    inbound_messages: crossbeam_queue::SegQueue<RawPacket>,
    outbound_messages_tx: mpsc::UnboundedSender<QueuedPacket>,
    socket: Arc<RaknetSocket>,
}

pub struct QueuedPacket {
    bytes: Bytes,
    reliability: Reliability,
    ordering_channel: u8,
}

impl Connection {
    pub fn new(socket: RaknetSocket) -> (Self, mpsc::UnboundedReceiver<QueuedPacket>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let connection = Self {
            inbound_messages: crossbeam_queue::SegQueue::new(),
            outbound_messages_tx: tx,
            socket: Arc::new(socket),
        };
        (connection, rx)
    }

    pub fn send<T: Packet + NetEncode>(
        &self,
        packet: T,
    ) -> Result<(), mpsc::error::SendError<QueuedPacket>> {
        let payload = packet.encode();
        let mut bytes = BytesMut::with_capacity(3 + payload.len());
        bytes.put_u8(PACKET_MARKER);
        bytes.put_i16(T::PACKET_ID);
        bytes.extend_from_slice(&payload);

        self.outbound_messages_tx.send(QueuedPacket {
            bytes: bytes.freeze(),
            reliability: T::RELIABILITY,
            ordering_channel: T::ORDERING_CHANNEL,
        })
    }

    pub fn receive(&self) -> Option<RawPacket> {
        self.inbound_messages.pop()
    }

    pub async fn start(
        self: Arc<Self>,
        mut outbound_messages: mpsc::UnboundedReceiver<QueuedPacket>,
    ) {
        let socket = self.socket.clone();
        let mut outgoing = tokio::spawn(async move {
            while let Some(packet) = outbound_messages.recv().await {
                if let Err(error) = socket.send_bytes_with_order_channel(packet.bytes, packet.reliability, packet.ordering_channel).await {
                    warn!("Failed to send packet: {error}");
                    break;
                }
            }
        });
        let connection = self.clone();
        let mut incoming = tokio::spawn(async move {
            loop {
                match connection.socket.recv_bytes().await {
                    Ok(bytes) => {
                        if let Some(packet) = RawPacket::read(bytes) {
                            connection.inbound_messages.push(packet);
                        }
                    }
                    Err(error) => {
                        info!("Connection receive ended: {error}");
                        break;
                    }
                }
            }
        });

        tokio::select! {
            result = &mut incoming => {
                if let Err(error) = result {
                    warn!("Incoming connection task failed: {error}");
                }
                outgoing.abort();
                let _ = outgoing.await;
            }
            result = &mut outgoing => {
                if let Err(error) = result {
                    warn!("Outgoing connection task failed: {error}");
                }
                incoming.abort();
                let _ = incoming.await;
            }
        }

        if let Err(error) = self.socket.close().await {
            warn!("Failed to close connection: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layline_packets::outbound::ping::OutboundPing;
    use layline_packets::outbound::rejected::Rejected;
    use raknet_v6::RaknetListener;
    use std::time::Duration;
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn receives_and_sends_without_waiting_for_more_input() {
        timeout(Duration::from_secs(10), async {
            let mut listener = RaknetListener::bind(&"127.0.0.1:0".parse().unwrap())
                .await
                .unwrap();
            let address = listener.local_addr().unwrap();
            listener.listen().await;
            let client = RaknetSocket::connect_with_version(&address, 6)
                .await
                .unwrap();
            let socket = listener.accept().await.unwrap();
            let (connection, outbound) = Connection::new(socket);
            let connection = Arc::new(connection);
            let task = tokio::spawn(connection.clone().start(outbound));

            client
                .send(b"\x86", Reliability::ReliableOrdered)
                .await
                .unwrap();
            client
                .send(b"\x86\x00\x72\x00\x01a", Reliability::ReliableOrdered)
                .await
                .unwrap();
            loop {
                if let Some(packet) = connection.receive() {
                    assert_eq!(packet.id, 0x0072);
                    assert_eq!(packet.bytes, b"\x00\x01a"[..]);
                    break;
                }
                sleep(Duration::from_millis(5)).await;
            }

            for connections in [1, 2] {
                connection.send(OutboundPing {
                    server_ip: "a".to_string(),
                    connections,
                    max_connections: 32,
                }).unwrap();
                let mut expected = b"\x86\x00\x72\x00\x01a".to_vec();
                expected.extend_from_slice(&i32::to_be_bytes(connections));
                expected.extend_from_slice(&32_i32.to_be_bytes());
                assert_eq!(client.recv().await.unwrap(), expected);
            }

            connection.send(Rejected {
                reason: "ServerFull".to_string(),
            }).unwrap();
            assert_eq!(
                client.recv().await.unwrap(),
                b"\x86\x00\x64\x00\x0aServerFull"
            );

            connection.socket.close().await.unwrap();
            task.await.unwrap();
            client.close().await.unwrap();
            listener.close().await.unwrap();
        })
        .await
        .expect("connection did not process messages or shut down");
    }
}
