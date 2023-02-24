Quic client-server app (python server, rust client)

How to run:

1. Install python package requirements with “python3.9 requirements.txt”
2. Run server with “python3.9 quic_server.py”
3. Run client with “cargo run” in git-quic-ping-pong/client-side/src/quic-client/src

How to test:
1. Test client with “cargo test -- --nocapture” in git-quic-ping-pong/client-side/src/quic-client/src
