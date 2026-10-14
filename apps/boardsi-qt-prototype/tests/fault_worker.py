#!/usr/bin/env python3
"""Test-only framed peer. Never used for the normal Rust integration case."""
import json
import os
import signal
import struct
import sys
import time


def read():
    size = struct.unpack('>I', sys.stdin.buffer.read(4))[0]
    return json.loads(sys.stdin.buffer.read(size))


first = read()
mode = sys.argv[1]
seq = 0


def emit(body, **override):
    global seq
    seq += 1
    msg = dict(version=1, run_id=first['run_id'],
               snapshot_id=first['snapshot_id'], seq=seq,
               kind='response' if body['type'] in ('Started', 'CancelAcknowledged') else 'event',
               body=body)
    msg.update(override)
    raw = json.dumps(msg, separators=(',', ':')).encode()
    frame = struct.pack('>I', len(raw)) + raw
    # Split early, periodic and terminal frames. Avoid imposing 600 sleep calls
    # on a platform whose background-process sleep can be heavily coalesced.
    if mode == 'partial' and (seq <= 4 or seq % 31 == 0 or body['type'] == 'Completed'):
        for cut in (frame[:2], frame[2:9], frame[9:]):
            sys.stdout.buffer.write(cut)
            sys.stdout.buffer.flush()
            time.sleep(0.01)
    else:
        sys.stdout.buffer.write(frame)
        sys.stdout.buffer.flush()


emit(dict(type='Started'))
if mode == 'crash':
    if os.name == 'nt':
        import ctypes
        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        kernel.GetCurrentProcess.restype = ctypes.c_void_p
        kernel.TerminateProcess.argtypes = [ctypes.c_void_p, ctypes.c_uint]
        kernel.TerminateProcess(kernel.GetCurrentProcess(), 0xC0000005)
    else:
        os.kill(os.getpid(), signal.SIGABRT)
if mode == 'exit7':
    sys.exit(7)
if mode.startswith('failed'):
    emit(dict(type='Failed', code='FixtureFailure', context='preserved context'))
    sys.exit(int(mode[-1]))
if mode == 'schema':
    emit(dict(type='Heartbeat'), version=2)
elif mode == 'duplicate':
    emit(dict(type='Heartbeat'), seq=1)
elif mode == 'reverse':
    emit(dict(type='Heartbeat'), seq=0)
elif mode == 'scope':
    emit(dict(type='Heartbeat'), run_id='previous-run')
elif mode == 'snapshot':
    emit(dict(type='Heartbeat'), snapshot_id='previous-snapshot')
elif mode == 'frame':
    sys.stdout.buffer.write(struct.pack('>I', 65537))
    sys.stdout.buffer.flush()
elif mode == 'json':
    sys.stdout.buffer.write(struct.pack('>I', 1) + b'{')
    sys.stdout.buffer.flush()
elif mode == 'truncated':
    sys.stdout.buffer.write(struct.pack('>I', 20) + b'{}')
    sys.stdout.buffer.flush()
    sys.exit(0)
elif mode == 'cancel-hang':
    read()
    emit(dict(type='CancelAcknowledged'))
    emit(dict(type='Cancelled'))
elif mode == 'no-heartbeat':
    for _ in range(12):
        emit(dict(type='Progress', done=0, total=100))
        time.sleep(0.5)
elif mode in ('completed-hang', 'partial', 'burst', 'completion-race', 'completion-after-ack', 'completion-incomplete', 'closed-cancel'):
    if mode == 'closed-cancel':
        os.close(0)
        print('CANCEL_PIPE_CLOSED', file=sys.stderr, flush=True)
        time.sleep(0.25)
    if mode == 'burst':
        for _ in range(20000):
            emit(dict(type='Heartbeat'))
    if mode.startswith('completion-'):
        read()
    if mode == 'completion-after-ack':
        emit(dict(type='CancelAcknowledged'))
    for point in range(99 if mode == 'completion-incomplete' else 100):
        emit(dict(type='Result', point=point, frequency_hz=point * 1e6,
                  s11=[0, 0], s21=[1, 0], synthetic=True))
        emit(dict(type='Progress', done=point+1, total=100))
    emit(dict(type='Completed'))
    if mode != 'completed-hang':
        sys.exit(0)
while True:
    time.sleep(1)
