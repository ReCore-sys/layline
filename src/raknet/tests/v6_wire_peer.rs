use std::{net::SocketAddr, time::Duration};

use raknet_v6::{RaknetSocket, Reliability};
use tokio::{net::UdpSocket, time::timeout};

const MAGIC: [u8; 16] = [
    0, 0xff, 0xff, 0, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78,
];

fn ipv4_address(address: SocketAddr) -> Vec<u8> {
    let SocketAddr::V4(address) = address else {
        panic!("expected IPv4");
    };
    let mut bytes = vec![4];
    bytes.extend(address.ip().octets().map(|octet| !octet));
    bytes.extend(address.port().to_be_bytes());
    bytes
}

fn ordered_frame(sequence: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = vec![0x80, sequence, 0, 0, 0x60];
    frame.extend(((payload.len() * 8) as u16).to_be_bytes());
    frame.extend([sequence, 0, 0, sequence, 0, 0, 0]);
    frame.extend(payload);
    frame
}

#[tokio::test]
async fn client_handshake_against_a_scripted_v6_wire_peer() {
    timeout(Duration::from_secs(5), async {
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = peer.local_addr().unwrap();
        let peer_task = tokio::spawn(async move {
            let mut buffer = [0; 2048];
            let (len, client) = peer.recv_from(&mut buffer).await.unwrap();
            assert_eq!(len, 1372);
            assert_eq!(&buffer[..18], &[&[5], &MAGIC[..], &[6]].concat());
            assert!(buffer[18..len].iter().all(|byte| *byte == 0));

            let mut reply = vec![6];
            reply.extend(MAGIC);
            reply.extend(7_u64.to_be_bytes());
            reply.push(0);
            reply.extend(1400_u16.to_be_bytes());
            peer.send_to(&reply, client).await.unwrap();

            let (len, source) = peer.recv_from(&mut buffer).await.unwrap();
            assert_eq!(source, client);
            assert_eq!(len, 34);
            assert_eq!(&buffer[..17], &[&[7], &MAGIC[..]].concat());
            assert_eq!(&buffer[17..24], ipv4_address(address));
            assert_eq!(&buffer[24..26], &1400_u16.to_be_bytes());
            let guid = buffer[26..34].to_vec();

            let mut reply = vec![8];
            reply.extend(MAGIC);
            reply.extend(7_u64.to_be_bytes());
            reply.extend(ipv4_address(client));
            reply.extend(1400_u16.to_be_bytes());
            reply.push(0);
            peer.send_to(&reply, client).await.unwrap();

            let (len, _) = peer.recv_from(&mut buffer).await.unwrap();
            assert!(len >= 32);
            assert_eq!(
                &buffer[..14],
                &[0x84, 0, 0, 0, 0x60, 0, 0xb0, 0, 0, 0, 0, 0, 0, 0]
            );
            assert_eq!(buffer[14], 9);
            assert_eq!(&buffer[15..23], guid);
            assert_eq!(buffer[31], 0);
            assert_eq!(&buffer[32..len], b"wire");
            let timestamp = buffer[23..31].to_vec();
            peer.send_to(&[0xc0, 0, 1, 1, 0, 0, 0], client)
                .await
                .unwrap();

            let mut accepted = vec![0x10];
            accepted.extend(ipv4_address(client));
            accepted.extend([0, 0]);
            for _ in 0..10 {
                accepted.extend([4, 255, 255, 255, 255, 0, 0]);
            }
            accepted.extend(timestamp);
            accepted.extend(1234_i64.to_be_bytes());
            peer.send_to(&ordered_frame(0, &accepted), client)
                .await
                .unwrap();

            loop {
                let (len, _) = peer.recv_from(&mut buffer).await.unwrap();
                if buffer[..len]
                    .windows(4)
                    .any(|bytes| bytes == [0x86, 1, 2, 3])
                {
                    break;
                }
            }
            peer.send_to(&ordered_frame(1, &[0x86, 4, 5, 6]), client)
                .await
                .unwrap();
        });

        let client = RaknetSocket::connect_with_password(&address, b"wire")
            .await
            .unwrap();
        client
            .send(&[0x86, 1, 2, 3], Reliability::ReliableOrdered)
            .await
            .unwrap();
        assert_eq!(client.recv().await.unwrap(), &[0x86, 4, 5, 6]);
        client.close().await.unwrap();
        peer_task.await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn listener_handshake_against_a_scripted_v6_wire_peer() {
    timeout(Duration::from_secs(5), async {
        let mut listener = raknet_v6::RaknetListener::bind(&"127.0.0.1:0".parse().unwrap())
            .await
            .unwrap()
            .with_password(b"wire")
            .unwrap();
        let address = listener.local_addr().unwrap();
        listener.listen().await;
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let client_address = peer.local_addr().unwrap();
        let mut request2 = vec![7];
        request2.extend(MAGIC);
        request2.extend(ipv4_address(address));
        request2.extend(1400_u16.to_be_bytes());
        request2.extend(42_u64.to_be_bytes());
        peer.send_to(&request2, address).await.unwrap();
        let mut buffer = [0; 2048];
        assert!(
            timeout(Duration::from_millis(50), peer.recv_from(&mut buffer))
                .await
                .is_err()
        );

        let mut request1 = vec![5];
        request1.extend(MAGIC);
        request1.push(6);
        request1.resize(1372, 0);
        peer.send_to(&request1, address).await.unwrap();
        let (len, _) = peer.recv_from(&mut buffer).await.unwrap();
        assert_eq!(len, 28);
        assert_eq!(&buffer[..17], &[&[6], &MAGIC[..]].concat());
        let guid = buffer[17..25].to_vec();
        assert_eq!(&buffer[25..28], &[0, 0x05, 0x78]);

        let mut oversized_request = request2.clone();
        oversized_request[24..26].copy_from_slice(&1401_u16.to_be_bytes());
        peer.send_to(&oversized_request, address).await.unwrap();
        assert!(
            timeout(Duration::from_millis(50), peer.recv_from(&mut buffer))
                .await
                .is_err()
        );

        peer.send_to(&request2, address).await.unwrap();
        let (len, _) = peer.recv_from(&mut buffer).await.unwrap();
        assert_eq!(len, 35);
        assert_eq!(&buffer[..17], &[&[8], &MAGIC[..]].concat());
        assert_eq!(&buffer[17..25], guid);
        assert_eq!(&buffer[25..32], ipv4_address(client_address));
        assert_eq!(&buffer[32..35], &[0x05, 0x78, 0]);

        let mut connected_request = vec![9];
        connected_request.extend(42_u64.to_be_bytes());
        connected_request.extend(1234_i64.to_be_bytes());
        connected_request.push(0);
        connected_request.extend(b"wire");
        peer.send_to(&ordered_frame(0, &connected_request), address)
            .await
            .unwrap();
        let accepted = loop {
            let (len, _) = peer.recv_from(&mut buffer).await.unwrap();
            if len > 14 && buffer[0] & 0xe0 == 0x80 && buffer[14] == 0x10 {
                break buffer[..len].to_vec();
            }
        };
        assert_eq!(&accepted[15..22], ipv4_address(client_address));
        assert_eq!(&accepted[24..94], &[4, 0, 0, 0, 0, 0, 0].repeat(10));
        assert_eq!(
            &accepted[accepted.len() - 16..accepted.len() - 8],
            &1234_i64.to_be_bytes()
        );
        peer.send_to(
            &[0xc0, 0, 1, 1, accepted[1], accepted[2], accepted[3]],
            address,
        )
        .await
        .unwrap();

        let mut incoming = vec![0x13];
        incoming.extend(ipv4_address(address));
        for _ in 0..10 {
            incoming.extend([4, 255, 255, 255, 255, 0, 0]);
        }
        incoming.extend(&accepted[accepted.len() - 16..]);
        peer.send_to(&ordered_frame(1, &incoming), address)
            .await
            .unwrap();
        peer.send_to(&ordered_frame(2, &[0x86, 7, 8, 9]), address)
            .await
            .unwrap();
        let socket = listener.accept().await.unwrap();
        assert_eq!(socket.recv().await.unwrap(), &[0x86, 7, 8, 9]);
        socket.close().await.unwrap();
        listener.close().await.unwrap();
    })
    .await
    .unwrap();
}
