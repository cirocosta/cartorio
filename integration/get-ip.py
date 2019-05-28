#!/usr/bin/python
import socket

def get_ip_address():
    """
    Retrieves the IP of the interface used in the main route
    for connecting to an external service.
    """
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.connect(("8.8.8.8", 80))

    return s.getsockname()[0]

if __name__ == "__main__":
    print get_ip_address()
