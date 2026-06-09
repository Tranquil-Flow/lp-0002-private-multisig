# LP-0002 Basecamp App

This directory contains a Logos Basecamp-loadable Qt plugin for the LP-0002 private multisig workflow.

It provides native plugin metadata, CMake, C++ backend sources, and QML assets so reviewers can inspect, build, and exercise the Basecamp package expected by Logos Basecamp.

## Build

```bash
cd basecamp-app
cmake -S . -B build -DCMAKE_BUILD_TYPE=Release
cmake --build build
```

The plugin exposes a QWidget wrapping `qml/Lp0002PrivateMultisig.qml` and a local `Lp0002Backend`.

The UI is an interactive evaluator surface:

- configure a threshold,
- edit proposal/action metadata,
- choose approvers,
- generate the public threshold journal,
- execute the threshold-gated action,
- inspect nullifiers, replay state, and the public audit journal.

The Basecamp app is intentionally local and deterministic so it remains safe to run during review. The root `demo.sh`, `scripts/demo-heavy-lane.sh`, and `submission/` evidence remain the executable proof and public LEZ trace sources.
