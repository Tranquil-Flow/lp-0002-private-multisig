#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP="$ROOT/basecamp-app"
cd "$APP"

for required in CMakeLists.txt metadata.json include/IComponent.h src/lp0002_backend.cpp src/lp0002_backend.h src/lp0002_plugin.cpp src/lp0002_plugin.h src/lp0002_widget.cpp src/lp0002_widget.h qml/Lp0002PrivateMultisig.qml resources.qrc; do
  test -f "$required" || { echo "FAIL: missing basecamp-app/$required" >&2; exit 1; }
done

export PATH="/opt/homebrew/bin:/opt/homebrew/opt/qt/bin:$PATH"
if ! command -v cmake >/dev/null 2>&1; then
  echo "PASS: native/QML Basecamp module structure present; cmake unavailable so build skipped"
  exit 0
fi

if [[ ! -d /opt/homebrew/opt/qt && -z "${CMAKE_PREFIX_PATH:-}" && -z "$(command -v qmake6 || true)" && -z "$(command -v qt-cmake || true)" ]]; then
  echo "PASS: native/QML Basecamp module structure present; Qt6 unavailable so build skipped"
  exit 0
fi

cmake -S . -B build -DCMAKE_PREFIX_PATH="${CMAKE_PREFIX_PATH:-/opt/homebrew/opt/qt}" -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel 4
test -f build/modules/liblp0002_private_multisig.dylib -o -f build/modules/liblp0002_private_multisig.so
echo "PASS: native/QML Basecamp module builds"
