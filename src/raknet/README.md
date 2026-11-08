This is an asynchronous RakNet protocol v6 backport of
[rust-raknet](https://github.com/b23r0/rust-raknet). It provides client and
listener APIs, reliability modes, fragmentation, and IPv4/IPv6 support. RakNet
protocol versions 10 and 11 are not supported, nor is libCat crypto stuff supported. PRs are welcome to add it if needed.

Listeners return an empty discovery payload by default. Applications can set
their own bytes with `RaknetListener::set_offline_data` and retrieve them with
`RaknetSocket::ping_bytes`. The former Bedrock-specific `set_motd` helper has
been removed; `set_full_motd` remains available for text payloads.