# Replay Python Script

Script for replaying SOE protocol version 3 packet captures.

## Prerequisites

* Python
* [uv](https://github.com/astral-sh/uv), which will automatically install all dependencies

## Usage

Point the client to the server 127.0.0.1:20260. Check the pcap file in Wireshark to determine the 
client's destination address. The script will replay all packets with this destination address to
the first client that connects to the UDP socket.

```bash
uv run ./replay.py --pcap capture.pcap --old-dest "192.168.1.145:59128" --new-src "127.0.0.1:20260"
```

## Known Issues

* The client eventually disconnects because this tool does not acknowledge any unexpected packets from the client.
