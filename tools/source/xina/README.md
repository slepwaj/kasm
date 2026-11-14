# Xina 拼音（带声调）

`xinapinin2.schema.yaml` / `xinapinin2.dict.yaml` 来自用户提供的 Rime 方案，作者 Xina；
说明见 <https://zhuanlan.zhihu.com/p/2092749558438744805>。仓库未声明额外许可证。

`tools/build-xina.py` 把词典转成 `assets/lexicon/xina.tsv`：声调数字 1-5 转成 `!@#$%`，
文本转简体，词频借用普通话词库。
