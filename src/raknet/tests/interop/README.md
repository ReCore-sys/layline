# RakNet v6 interoperability

The self-contained `tests/v6_wire_peer.rs` tests use a scripted UDP peer with
independently constructed v6 handshake bytes. They do not replace testing
against a separate RakNet v6 implementation, particularly in secure mode.
