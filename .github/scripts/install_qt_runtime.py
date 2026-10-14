"""Install the same Qt 6.8.3 qtbase SDK; identify target and host prefixes."""
import os
from pathlib import Path
import subprocess
import sys

host, arch = sys.argv[1:]
root = Path(os.environ['RUNNER_TEMP']) / 'qt-native'


def install(architecture):
    subprocess.run([sys.executable, '-m', 'aqt', 'install-qt', host, 'desktop',
                    '6.8.3', architecture, '--archives', 'qtbase', '-O', str(root)], check=True)


host_prefix = None
if 'arm64' in arch and host == 'windows':
    install('win64_msvc2022_64')  # Host SDK tools may run under Windows ARM emulation.
    host_prefix = root / '6.8.3' / 'msvc2022_64'
install(arch)
configs = list((root / '6.8.3').glob('*/lib/cmake/Qt6/Qt6Config.cmake'))
prefixes = [p.parents[3] for p in configs]
if host_prefix is not None:
    prefixes = [p for p in prefixes if p != host_prefix]
if len(prefixes) != 1:
    raise RuntimeError(f'Ambiguous target Qt prefix: {prefixes}')
prefix = prefixes[0]
with Path(os.environ['GITHUB_ENV']).open('a') as f:
    f.write(f'QT_PREFIX={prefix.as_posix()}\n')
    if host_prefix is not None:
        f.write(f'QT_HOST_PREFIX={host_prefix.as_posix()}\n')
with Path(os.environ['GITHUB_PATH']).open('a') as f:
    f.write(str(prefix / 'bin') + '\n')
print('Target SDK:', prefix, 'Host tools:', host_prefix)
