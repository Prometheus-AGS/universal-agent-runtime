"""Fixed controlled upstream entrypoint inside the preinstalled qualification Pod.

The source-bound runner alone starts it after its full-phase boundary. This peer
has no UAR execution, approval, persistence, or recovery authority.
"""
import argparse
import hashlib
import os
from pathlib import Path
import secrets


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--port', type=int, required=True)
    parser.add_argument('--trace', required=True)
    parser.add_argument('--provider-sha256', required=True)
    parser.add_argument('--control-token-file', required=True)
    args = parser.parse_args()
    assert hashlib.sha256(Path(__file__).with_name('provider.py').read_bytes()).hexdigest() == args.provider_sha256
    assert args.trace.startswith('/var/lib/uar/dot17/') and '..' not in Path(args.trace).parts
    assert args.control_token_file.startswith('/var/lib/uar/dot17/') and '..' not in Path(args.control_token_file).parts
    assert 1024 <= args.port <= 65535
    Path(args.trace).parent.mkdir(parents=True, exist_ok=True)
    with open(args.trace, 'x'):
        pass
    token = secrets.token_hex(32)
    descriptor = os.open(args.control_token_file, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, 'w') as output:
        output.write(token)
        output.flush()
        os.fsync(output.fileno())
    from provider import Provider
    peer = Provider(args.port, args.trace, control_token=token, bind_host='0.0.0.0')
    try:
        peer.server.serve_forever()
    finally:
        peer.server.server_close()
        Path(args.control_token_file).unlink(missing_ok=True)


if __name__ == '__main__':
    main()
