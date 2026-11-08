# KASM 鸿蒙壳（PoC）

HarmonyOS NEXT 的输入法扩展壳：ArkTS 输入法扩展 + ArkUI 键盘 + Rust 引擎（NAPI）。

当前进度：带输入法扩展的 HAP 能构建；键盘可组句、出候选（30 条，9 条一页，可翻页）、
点选或空格上屏、退格；顶部可循环切换 全拼 / 粤拼 / 通字 / tupa / 上古音，五套词库随包解到沙箱。
还没有做：正式签名、真机验证、候选条位图、完整设置页。

## 结构

```
apps/harmony/
  core/                       Rust cdylib/staticlib（C ABI + JSON 帧）
  entry/                      HarmonyOS 模块
    src/main/ets/inputmethod/  InputMethodExtensionAbility（建 panel）
    src/main/ets/pages/        Keyboard 键盘页（NAPI 调引擎）
    src/main/cpp/              NAPI C++ 封装 + CMake
    src/main/resources/rawfile/  随包词库（构建时拷贝）
    libs/arm64-v8a/             Rust 静态库（构建时拷贝）
```

## 依赖

- DevEco Studio 6.0 / HarmonyOS SDK **API 20**（`DEVECO_SDK_HOME` 指向其 `sdk`）
- Rust **1.99+**，并装好目标：

```powershell
rustup target add aarch64-unknown-linux-ohos
```

> 仓库 `rust-toolchain.toml` 钉的是 1.96.0；如果这个版本没有 OHOS std，构建时用
> `RUSTUP_TOOLCHAIN=1.99.1` 覆盖。

## 构建

```powershell
# 1) Rust 静态库
$env:RUSTUP_TOOLCHAIN = '1.99.1'
cargo build -p kasm-harmony-core --target aarch64-unknown-linux-ohos --release

# 2) 放静态库与词库
New-Item -ItemType Directory -Force entry/libs/arm64-v8a | Out-Null
Copy-Item target/aarch64-unknown-linux-ohos/release/libkasm_harmony_core.a entry/libs/arm64-v8a/
Copy-Item assets/lexicon/dict.tsv entry/src/main/resources/rawfile/dict.tsv

# 3) 构建 HAP（hvigorw 拒绝非 ASCII 路径，仓库路径含中文时要先复制到 ASCII 路径）
$env:NODE_HOME = 'D:\Huawei\DevEco Studio\tools\node'
$env:DEVECO_SDK_HOME = 'D:\Huawei\DevEco Studio\sdk'
& 'D:\Huawei\DevEco Studio\tools\hvigor\bin\hvigorw.bat' --mode module -p product=default -p module=entry@default assembleHap
# 产物：entry/build/default/outputs/default/entry-default-unsigned.hap
```

`build.ps1` 把以上步骤串起来了（含自动镜像到 `%TEMP%\kasm-harmony-build`）。

'## 模拟器

DevEco 的本地模拟器在 Windows 上是 **x86_64** 镜像，所以 HAP 需要同时带 `x86_64`：

- Rust 侧：`rustup target add x86_64-unknown-linux-ohos`，再 `cargo build -p kasm-harmony-core --target x86_64-unknown-linux-ohos --release`；
- `entry/libs/x86_64/libkasm_harmony_core.a`，CMake 按 `${OHOS_ARCH}` 取对应目录；
- `entry/build-profile.json5` 的 `abiFilters` 已含 `arm64-v8a` 与 `x86_64`。

模拟器镜像本身要在 DevEco 的 Device Manager 里下载（本机只装了 `Emulator.exe`，没有系统镜像），且安装 HAP 仍需签名。

## 运行前提'

- HAP 是 **unsigned** 的，安装前要在 DevEco 里配 `signingConfigs`（自动签名或
  `build-profile.json5` 手填 p12/cer/p7b），或者用 DevEco 的「Run」直接装。
- 输入法要在「设置 → 系统 → 输入法」里添加并启用；模拟器对 IME 支持有限，建议真机。
- 主词库解到沙箱 `filesDir/dict.tsv`，首次加载后 mmap。

## 下一步

1. 签名并在真机跑通「选 KASM → 敲 ni → 候选 你 → 空格上屏」。
2. 候选条改成 `kasm-render` 位图（PixelMap），支持翻页、数字选词。
3. 设置页：拼音方案（全拼/粤拼/通字/tupa/上古音）、简繁、中英。
4. 把 Android 的 `Session` 抽成共享 `crates/kasm-ime`，两壳共用。
