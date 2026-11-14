# Xina 拼音（带声调）

`xinapinin2.schema.yaml` / `xinapinin2.dict.yaml` 来自用户提供的 Rime 方案，作者 Xina；
说明见 <https://zhuanlan.zhihu.com/p/2092749558438744805>。仓库未声明额外许可证。

`tools/build-xina.py` 把词典转成 `assets/lexicon/xina.tsv`：声调数字 1-5 转成 `!@#$%`，
文本转简体，词频借用普通话词库。

# Xina 切拼（中古音）

`常用字表_Xina切拼.xlsx` 来自用户提供的中古音字表（三列：字 / 中古音 / 來源）；
`tools/build-xina-qiepin.py` 转成 `assets/lexicon/xina-qiepin.tsv` 与
`crates/kasm-core/data/xina-qiepin-syllables.txt`。
