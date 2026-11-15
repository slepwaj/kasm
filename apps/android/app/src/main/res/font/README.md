# TIPA 键盘字体

`tipa.ttf` 由 `tools/build-tipa-font.py` 从工作区 `tipa/` 的 TIPA Type1 字体转换：
用 `t3enc.def` 的 T3 槽位和 `tipauni.dtx` 的命令→Unicode 对照生成 cmap，再用
`fontTools` 转成 Android 可加载的 TTF。

TIPA 原字体按 LPPL 授权；`tipauni.dtx` 按 GPL v3 授权。只用于 IPA / unt / 中文通用羅馬字键盘的键面。
