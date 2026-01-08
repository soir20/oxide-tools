#!/usr/bin/python

# /// script
# dependencies = [
#   "scapy",
#   "tqdm",
# ]
# ///

from scapy.all import *
from tqdm import tqdm
import argparse
import socket
import sys
import time

def parse_args():
    parser = argparse.ArgumentParser(
        description="Replay a PCAP while rewriting destination and source IP:PORT"
    )

    parser.add_argument(
        "--pcap",
        required=True,
        help="Path to the PCAP file"
    )

    parser.add_argument(
        "--old-dest",
        required=True,
        help="Old destination in form IP:PORT (the one inside the PCAP)"
    )

    parser.add_argument(
        "--new-src",
        required=True,
        help="New source in form IP:PORT (what packets should appear to come from)"
    )

    return parser.parse_args()


def should_send_packet(packet, old_destination_ip, old_destination_port):
    if not packet.haslayer(IP) or not packet.haslayer(UDP):
        return False

    if packet[IP].dst != old_destination_ip or packet.dport != old_destination_port:
        return False

    return True


def main():
    args = parse_args()

    packets = rdpcap(args.pcap)
    old_destination = args.old_dest
    new_source = args.new_src

    old_destination_ip, old_destination_port_str = old_destination.split(":")
    new_source_ip, new_source_port_str = new_source.split(":")

    old_destination_port = int(old_destination_port_str)
    new_source_port = int(new_source_port_str)

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.bind((new_source_ip, new_source_port))

    session_req, (new_destination_ip, new_destination_port) = sock.recvfrom(512)
    session_id = session_req[6:10]

    filtered_packets = [packet for packet in packets if should_send_packet(packet, old_destination_ip, old_destination_port)]

    if len(filtered_packets) == 0:
        print("No packets to send")
        return

    filtered_packets[0][Raw].load = filtered_packets[0][Raw].load[:2] + session_id + filtered_packets[0][Raw].load[6:]

    clock = filtered_packets[0].time
    for packet in tqdm(filtered_packets, "Sending packets"):
        time.sleep(max(float(packet.time - clock), 0))
        clock = packet.time

        sock.sendto(packet[Raw].load, (new_destination_ip, new_destination_port))


if __name__ == "__main__":
    main()