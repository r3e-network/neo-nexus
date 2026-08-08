#!/usr/bin/env python3
"""Accept on emulator ports and immediately close — makes adb scan fail fast."""
import socket, time, select

SOCKETS = []
for port in range(5554, 5586):
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("127.0.0.1", port))
        s.listen(1)
        SOCKETS.append(s)
    except OSError:
        pass

print(f"Port-blocker: holding {len(SOCKETS)} ports (5554-5585)", flush=True)
while True:
    readable, _, _ = select.select(SOCKETS, [], [], 10)
    for s in readable:
        try:
            conn, addr = s.accept()
            conn.close()
        except:
            pass
