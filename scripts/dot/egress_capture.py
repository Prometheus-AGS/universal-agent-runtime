"""Independent packet capture for default-profile egress qualification.

Requires a dedicated quiet qualification host, tcpdump capture permission, and
an Ethernet/SLL/loopback capture interface. Unsupported capture formats block.
Options follow the installed tcpdump(1) manual; no shell is invoked.
"""
import ipaddress
import json
import os
from pathlib import Path
import re
import shutil
import signal
import socket
import struct
import subprocess
import time

from acceptance import Blocked, require


class Capture:
    def __init__(self, config, evidence):
        require(config['dedicated_qualification_host'] is True, 'capture requires an isolated qualification host')
        self.config = config
        self.evidence = evidence
        self.path = evidence.directory / 'default-egress.pcap'
        self.log = evidence.directory / 'default-egress-capture.log'
        self.process = None
        self.stderr = None

    def start(self):
        binary = shutil.which('tcpdump')
        if binary is None:
            raise Blocked('tcpdump is unavailable; absence of remote egress remains unverified')
        self.stderr = open(self.log, 'xb')
        self.process = subprocess.Popen([binary, '-U', '-n', '-i', self.config['interface'],
            '-s', '128', '-w', str(self.path)], stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL, stderr=self.stderr)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise Blocked('packet collector failed; capture permission or interface unavailable')
            if self.path.exists() and self.path.stat().st_size >= 24:
                break
            time.sleep(.1)
        else:
            raise Blocked('packet capture was not ready before runtime boot')
        control = self.config['capture_control']
        # Positive wire control proves this interface observes outbound traffic.
        with socket.create_connection((control['ip'], control['port']), timeout=10):
            pass

    def finish(self):
        if self.process is None:
            raise Blocked('packet collector was never started')
        self.process.send_signal(signal.SIGINT)
        self.process.wait(timeout=15)
        self.stderr.close()
        text = self.log.read_text(errors='replace')
        dropped = re.findall(r'(\d+) packets dropped by kernel', text)
        require(dropped and int(dropped[-1]) == 0, 'capture loss prevents absence-of-egress proof')
        packets = list(read_packets(self.path))
        local = {ipaddress.ip_address(value) for value in self.config['local_addresses']}
        allowed = {(ipaddress.ip_address(value['ip']), value['port']) for value in self.config['allowed_infrastructure']}
        control = self.config['capture_control']
        control_key = (ipaddress.ip_address(control['ip']), control['port'])
        require(control_key in allowed, 'positive capture control must be explicitly allowed')
        outgoing = [packet for packet in packets if packet[0] in local]
        require(any((packet[1], packet[3]) == control_key for packet in outgoing),
                'positive control traffic was not captured; wrong interface/address inventory')
        unexpected = [packet for packet in outgoing if not packet[1].is_loopback
                      and (packet[1], packet[3]) not in allowed]
        require(not unexpected, 'unconfigured profile emitted non-infrastructure egress: ' + str(unexpected))
        self.evidence.observe('default-no-remote-egress', {'capture': str(self.path),
            'outbound_packets': len(outgoing), 'unexpected_outbound_packets': 0, 'dropped_packets': 0,
            'local_addresses': self.config['local_addresses'],
            'allowed_infrastructure': self.config['allowed_infrastructure'], 'positive_control_seen': True})


def read_packets(path):
    with open(path, 'rb') as stream:
        header = stream.read(24)
        require(len(header) == 24, 'truncated packet capture')
        if header[:4] in (b'\xd4\xc3\xb2\xa1', b'\x4d\x3c\xb2\xa1'):
            endian = '<'
        elif header[:4] in (b'\xa1\xb2\xc3\xd4', b'\xa1\xb2\x3c\x4d'):
            endian = '>'
        else:
            raise Blocked('capture format is not supported classic pcap')
        link = struct.unpack(endian + 'I', header[20:24])[0]
        while packet_header := stream.read(16):
            require(len(packet_header) == 16, 'truncated packet header')
            captured = struct.unpack(endian + 'IIII', packet_header)[2]
            packet = stream.read(captured)
            require(len(packet) == captured, 'truncated packet bytes')
            if link == 1:
                require(len(packet) >= 14, 'short Ethernet frame')
                protocol, offset = struct.unpack('!H', packet[12:14])[0], 14
                while protocol in (0x8100, 0x88a8):
                    protocol = struct.unpack('!H', packet[offset+2:offset+4])[0]
                    offset += 4
            elif link == 113:
                protocol, offset = struct.unpack('!H', packet[14:16])[0], 16
            elif link == 276:
                protocol, offset = struct.unpack('!H', packet[:2])[0], 20
            elif link in (0, 108):
                offset = 4
                protocol = 0x86dd if packet[offset] >> 4 == 6 else 0x0800
            else:
                raise Blocked('unsupported packet capture link type: ' + str(link))
            if protocol == 0x0806:
                continue  # ARP has no remote application payload.
            if protocol == 0x0800:
                require(len(packet) >= offset + 20, 'short IPv4 packet')
                src = ipaddress.ip_address(packet[offset+12:offset+16])
                dst = ipaddress.ip_address(packet[offset+16:offset+20])
                transport = packet[offset+9]
                transport_offset = offset + (packet[offset] & 15) * 4
                require(struct.unpack('!H', packet[offset+6:offset+8])[0] & 0x1fff == 0,
                        'fragmented traffic prevents port-level egress attribution')
            elif protocol == 0x86dd:
                require(len(packet) >= offset + 40, 'short IPv6 packet')
                src = ipaddress.ip_address(packet[offset+8:offset+24])
                dst = ipaddress.ip_address(packet[offset+24:offset+40])
                transport = packet[offset+6]
                transport_offset = offset + 40
                if transport == 58 and (dst.is_multicast or dst.is_link_local):
                    continue  # Link-local neighbor/router discovery.
                require(transport in (6, 17), 'IPv6 extension traffic requires a qualified decoder')
            else:
                continue  # Non-IP link traffic is outside inference egress.
            if transport not in (6, 17):
                yield src, dst, transport, None
                continue
            require(len(packet) >= transport_offset + 4, 'snap length omitted destination port')
            port = struct.unpack('!H', packet[transport_offset+2:transport_offset+4])[0]
            yield src, dst, transport, port
