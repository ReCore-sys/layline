use std::time::Duration;

use raknet_v6::{RaknetListener, RaknetSocket, Reliability, error::RaknetError};
use tokio::{net::UdpSocket, time::timeout};

#[tokio::test]
async fn discovery_preserves_binary_and_empty_offline_data() {
    timeout(Duration::from_secs(5), async {
        let mut listener = RaknetListener::bind(&"127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        listener.listen().await;
        assert_eq!(RaknetSocket::ping_bytes(&address).await.unwrap().1, b"");

        let data = vec![0, 0xff, 0xfe, 1, 0];
        listener.set_offline_data(data.clone()).await.unwrap();
        assert_eq!(listener.get_offline_data().await, data);
        assert_eq!(RaknetSocket::ping_bytes(&address).await.unwrap().1, data);
        assert!(matches!(
            RaknetSocket::ping(&address).await,
            Err(RaknetError::PacketParseError)
        ));
        listener.close().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn discovery_data_configured_before_listening_is_preserved() {
    timeout(Duration::from_secs(5), async {
        let mut listener = RaknetListener::bind(&"127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let address = listener.local_addr().unwrap();
        let data = vec![0, 0x86, 0xff];
        listener.set_offline_data(data.clone()).await.unwrap();
        listener.listen().await;
        assert_eq!(RaknetSocket::ping_bytes(&address).await.unwrap().1, data);
        listener.close().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn passwords_and_application_message_ids_work_in_both_directions() {
    timeout(Duration::from_secs(5), async {
        let mut listener = RaknetListener::bind(&"127.0.0.1:0".parse().unwrap())
            .await
            .unwrap()
            .with_password(b"password\0bytes")
            .unwrap();
        let address = listener.local_addr().unwrap();
        listener.listen().await;

        assert!(matches!(
            RaknetSocket::connect_with_password(&address, b"wrong").await,
            Err(RaknetError::InvalidPassword)
        ));
        let client = RaknetSocket::connect_with_password(&address, b"password\0bytes")
            .await
            .unwrap();
        let server = listener.accept().await.unwrap();
        for id in [0x55, 0x80, 0x86, 0x90, 0xa0, 0xc0, 0xfe] {
            let message = [id, 1, 2, 3];
            client
                .send(&message, Reliability::ReliableOrdered)
                .await
                .unwrap();
            assert_eq!(server.recv().await.unwrap(), message);
            server
                .send(&message, Reliability::ReliableOrdered)
                .await
                .unwrap();
            assert_eq!(client.recv().await.unwrap(), message);
        }
        client.close().await.unwrap();
        server.close().await.unwrap();
        listener.close().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn version_and_password_limits_are_checked_before_connecting() {
    let address = "127.0.0.1:1".parse().unwrap();
    for version in [10, 11] {
        assert!(matches!(
            RaknetSocket::connect_with_version(&address, version).await,
            Err(RaknetError::NotSupportVersion)
        ));
    }
    assert!(matches!(
        RaknetSocket::connect_with_password(&address, &[0; 256]).await,
        Err(RaknetError::PasswordTooLong)
    ));
}

#[tokio::test]
async fn maximum_length_password_is_accepted() {
    timeout(Duration::from_secs(5), async {
        let password = [0x5a; 255];
        let mut listener = RaknetListener::bind(&"127.0.0.1:0".parse().unwrap())
            .await
            .unwrap()
            .with_password(&password)
            .unwrap();
        let address = listener.local_addr().unwrap();
        listener.listen().await;
        let client = RaknetSocket::connect_with_password(&address, &password)
            .await
            .unwrap();
        let server = listener.accept().await.unwrap();
        client.close().await.unwrap();
        server.close().await.unwrap();
        listener.close().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn rejects_secure_reply_instead_of_attempting_insecure_handshake() {
    timeout(Duration::from_secs(3), async {
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = peer.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut request = [0; 2048];
            let (len, client) = peer.recv_from(&mut request).await.unwrap();
            assert!(len >= 18);
            assert_eq!(request[0], 0x05);
            assert_eq!(request[17], 6);
            let mut reply = vec![0x06];
            reply.extend_from_slice(&request[1..17]);
            reply.extend_from_slice(&1_u64.to_be_bytes());
            reply.push(1);
            reply.extend_from_slice(&[0; 70]);
            peer.send_to(&reply, client).await.unwrap();
        });
        assert!(matches!(
            RaknetSocket::connect(&address).await,
            Err(RaknetError::SecurityUnsupported)
        ));
        task.await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn rejects_inconsistent_server_guid_during_offline_handshake() {
    timeout(Duration::from_secs(3), async {
        let peer = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let address = peer.local_addr().unwrap();
        let task = tokio::spawn(async move {
            let mut request = [0; 2048];
            let (_, client) = peer.recv_from(&mut request).await.unwrap();
            let magic = request[1..17].to_vec();
            let mut reply = vec![6];
            reply.extend(&magic);
            reply.extend(1_u64.to_be_bytes());
            reply.push(0);
            reply.extend(1400_u16.to_be_bytes());
            peer.send_to(&reply, client).await.unwrap();

            peer.recv_from(&mut request).await.unwrap();
            let std::net::SocketAddr::V4(client) = client else {
                panic!("expected IPv4");
            };
            let mut reply = vec![8];
            reply.extend(&magic);
            reply.extend(2_u64.to_be_bytes());
            reply.push(4);
            reply.extend(client.ip().octets().map(|octet| !octet));
            reply.extend(client.port().to_be_bytes());
            reply.extend(1400_u16.to_be_bytes());
            reply.push(0);
            peer.send_to(&reply, client).await.unwrap();
        });
        assert!(matches!(
            RaknetSocket::connect(&address).await,
            Err(RaknetError::IncorrectReply)
        ));
        task.await.unwrap();
    })
    .await
    .unwrap();
}
