# LP-0002 Basecamp native module build evidence

The static browser preview in `basecamp-app/index.html` is now complemented by a Logos Basecamp-style Qt plugin package.

Files added:

- `basecamp-app/CMakeLists.txt`
- `basecamp-app/metadata.json`
- `basecamp-app/include/IComponent.h`
- `basecamp-app/src/lp0002_plugin.{h,cpp}`
- `basecamp-app/src/lp0002_widget.{h,cpp}`
- `basecamp-app/qml/Lp0002PrivateMultisig.qml`
- `basecamp-app/resources.qrc`

Verified build command on the M4 Pro:

```bash
cd basecamp-app
export PATH=/opt/homebrew/bin:/opt/homebrew/opt/qt/bin:$PATH
cmake -S . -B build -DCMAKE_PREFIX_PATH=/opt/homebrew/opt/qt -DCMAKE_BUILD_TYPE=Release
cmake --build build --parallel 4
```

Observed result:

```text
[100%] Built target lp0002_private_multisig
```

Built plugin artifact:

```text
basecamp-app/build/modules/liblp0002_private_multisig.dylib
```

Honesty note: this proves the native/QML plugin package builds locally against Qt 6 and exposes the Basecamp `IComponent` plugin interface. It does **not** prove final LogosBasecamp runtime load/activation. Final publication must attach `submission/BASECAMP_RUNTIME_LOAD_EVIDENCE.json` with raw runtime logs, log hashes, a loaded component id, and `final_basecamp_runtime_load_evidence=true`.
