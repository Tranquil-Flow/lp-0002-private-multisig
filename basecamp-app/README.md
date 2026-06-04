# LP-0002 Basecamp App

This directory contains a Logos Basecamp-loadable Qt plugin shell for the LP-0002 private multisig workflow.

It provides native plugin metadata, CMake, C++ sources, and QML assets so reviewers can inspect/build the package shape expected by Logos Basecamp.

## Build

```bash
cd basecamp-app
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

The plugin exposes a QWidget wrapping `qml/Lp0002PrivateMultisig.qml`. The UI is intentionally local/offline: it explains the five-step flow and points reviewers to the root `demo.sh`, `scripts/demo-heavy-lane.sh`, and `submission/` evidence for executable proof and LEZ traces.
