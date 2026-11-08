use bytes::Bytes;
use layline_codec::{NetDecode, Packet};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use tracing::{trace, warn};

type Handler<W, S> = Box<dyn Fn(&mut W, &S, &mut dyn Any)>;

struct Entry<W, S> {
    name: &'static str,
    type_id: TypeId,
    decode: fn(&mut Bytes) -> Option<Box<dyn Any>>,
    handlers: Vec<Handler<W, S>>,
}

/// Routes inbound packets to the handlers registered for their type.
///
/// `W` is the world handlers mutate and `S` identifies where the packet came from.
pub struct Registry<W, S> {
    entries: HashMap<i16, Entry<W, S>>,
}

impl<W: 'static, S: 'static> Registry<W, S> {
    /// Creates a registry that can decode every inbound packet but has no handlers yet.
    pub fn new() -> Self {
        let mut registry = Self {
            entries: HashMap::new(),
        };
        register_inbound(&mut registry);
        registry
    }

    fn add_decoder<P: Packet + NetDecode + 'static>(&mut self) {
        let entry = Entry {
            name: P::PACKET_NAME,
            type_id: TypeId::of::<P>(),
            decode: |bytes| P::decode(bytes).map(|packet| Box::new(packet) as Box<dyn Any>),
            handlers: Vec::new(),
        };
        let previous = self.entries.insert(P::PACKET_ID, entry);
        assert!(
            previous.is_none(),
            "duplicate inbound packet id 0x{:04X}",
            P::PACKET_ID
        );
    }

    /// Adds a handler for `P`. Handlers run in registration order.
    ///
    /// # Panics
    ///
    /// Panics if `P` is not an inbound packet.
    pub fn on<P: Packet + 'static>(&mut self, handler: fn(&mut W, &S, &mut P)) {
        let entry = self
            .entries
            .get_mut(&P::PACKET_ID)
            .filter(|entry| entry.type_id == TypeId::of::<P>())
            .unwrap_or_else(|| panic!("`{}` is not an inbound packet", P::PACKET_NAME));
        entry.handlers.push(Box::new(move |world, source, packet| {
            let packet = packet
                .downcast_mut::<P>()
                .expect("packet type matches its registered id");
            handler(world, source, packet);
        }));
    }

    /// Decodes the packet with the given id and passes it to its handlers.
    pub fn dispatch(&self, world: &mut W, source: &S, id: i16, mut bytes: Bytes) {
        let Some(entry) = self.entries.get(&id) else {
            warn!("Unknown packet ID: 0x{:04X}", id);
            return;
        };
        if entry.handlers.is_empty() {
            trace!("No handlers registered for {}", entry.name);
            return;
        }
        let Some(mut packet) = (entry.decode)(&mut bytes) else {
            warn!("Failed to decode {} packet", entry.name);
            return;
        };
        for handler in &entry.handlers {
            handler(world, source, &mut *packet);
        }
    }
}

impl<W: 'static, S: 'static> Default for Registry<W, S> {
    fn default() -> Self {
        Self::new()
    }
}

include!(concat!(env!("OUT_DIR"), "/inbound.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::{login::Login, ping::InboundPing};
    use crate::outbound::ping::OutboundPing;
    use layline_codec::NetEncode;

    type TestRegistry = Registry<Vec<String>, ()>;

    fn ping_bytes(server_ip: &str) -> Bytes {
        server_ip.encode()
    }

    #[test]
    fn dispatches_to_handlers_in_order() {
        let mut registry = TestRegistry::new();
        registry.on(|world, _, packet: &mut InboundPing| {
            world.push(format!("first {}", packet.server_ip));
            packet.server_ip.push('!');
        });
        registry.on(|world, _, packet: &mut InboundPing| {
            world.push(format!("second {}", packet.server_ip));
        });

        let mut world = Vec::new();
        registry.dispatch(&mut world, &(), InboundPing::PACKET_ID, ping_bytes("a"));
        assert_eq!(world, ["first a", "second a!"]);
    }

    #[test]
    fn ignores_unknown_undecodable_and_unhandled_packets() {
        let mut registry = TestRegistry::new();
        registry.on(|world, _, _: &mut InboundPing| world.push("ping".to_string()));

        let mut world = Vec::new();
        registry.dispatch(&mut world, &(), 0x7fff, Bytes::new());
        registry.dispatch(
            &mut world,
            &(),
            InboundPing::PACKET_ID,
            Bytes::from_static(b"\x00"),
        );
        registry.dispatch(&mut world, &(), Login::PACKET_ID, Bytes::new());
        assert!(world.is_empty());
    }

    #[test]
    #[should_panic(expected = "is not an inbound packet")]
    fn rejects_handlers_for_outbound_packets() {
        // Shares its id with the inbound ping.
        TestRegistry::new().on(|_, _, _: &mut OutboundPing| {});
    }
}
