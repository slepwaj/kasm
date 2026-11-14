# Xina 拼音

当前方案以 `Xina拼音.docx` 的《Xina传统派普通话拼音方案》为准：声母表、韵母表和声调表都在文档里。
`tools/build-xina.py` 把普通话词库的拼音转成 Xina 拼式，并给单字/词的常见读音补
`!@#$%` 声调变体，生成 `assets/lexicon/xina.tsv` 与 `crates/kasm-core/data/xina-syllables.txt`。

旧版 Rime `xinapinin2.schema.yaml` / `xinapinin2.dict.yaml` 留在本目录只作历史参考，
当前构建不再使用；`常用字表_Xina切拼.xlsx` 是另一套 Xina 切拼（中古音），也不参与 Xina 拼音。
