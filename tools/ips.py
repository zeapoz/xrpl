# Tools for creating virtual linux dummy devices that can be addressed, addresses those devices with
# hardware address, ip address and sets the link up. Moreover, it generates list of valid addresses (where
# ip/ipconfig command invocation completed successfully) and writes that list to the Rust file.
#
# Only Linux is supported! Script need to be run with sudo.
#
# User can tweak settings using command line.
# Just run python ips.py --help for more information about parameters.


import argparse
import ipaddress
import os
import random
import sys


def generate_hwaddr():
    return "02:00:00:%02x:%02x:%02x" % (random.randint(0, 255),
                                        random.randint(0, 255),
                                        random.randint(0, 255))


def generate_dev(device_name, ip_addr):
    cmd = 'ip link add ' + device_name + ' type dummy && '
    cmd += 'ifconfig ' + device_name + ' hw ether ' + generate_hwaddr() + ' && '
    cmd += 'ip addr add ' + ip_addr + ' dev ' + device_name + ' && '
    cmd += 'ip link set ' + device_name + ' up'
    return os.system(cmd)


parser = argparse.ArgumentParser(description='Setting interfaces and generating Rust file with IPs.')
parser.add_argument('--subnet', nargs='?', default='1.1.1.0/28', help='Subnet to generate', type=str)
parser.add_argument('--file', nargs='?', default='ips.rs', help='Output file with IPs', type=str)
parser.add_argument('--dev_prefix', nargs='?', default='test_eth', help='How to prefix dummy devs', type=str)

args = parser.parse_args()

subnet = args.subnet
file = args.file
dev = args.dev_prefix

ips = []

for ip in ipaddress.IPv4Network(subnet):
    ips.append(ip)

if not sys.platform.startswith('linux'):
    print('Setting virtual interfaces can be done only on Linux')
    sys.exit(0)

i = 0

with open(file, 'w') as f:
    f.write('pub const IPS: &\'static [&\'static str] = &[ \n')
    for ip in ips:
        if generate_dev(dev + str(i), str(ip)) == 0:
            f.write('\t"' + str(ip) + '",\n')
        i += 1

    f.write('];\n\n')
