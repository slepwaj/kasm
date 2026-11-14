# 吳語（混合腔）rime-wugniu-suwu

`wugniu_suwu.schema.yaml` / `wugniu_suwu.dict.yaml` 来自用户提供的 Rime 方案
（雪螢 SiqYin、予安等，见 <https://www.wugniu.com/> 与
<https://github.com/NGLI/rime-wugniu_soutseu/>）；仓库未声明额外许可证。

`tools/build-wugniu.py` 去掉读音末尾的声调数字、文本转简体后生成
`assets/lexicon/wugniu.tsv`，音节表在 `crates/kasm-core/data/wugniu-syllables.txt`。

`fuzzy.tsv` 是这个方案自己的等价拼写规则（来自 Rime schema 的 `derive` 段），
构建时展开进词库和音节表；只作用于吴语方案，不影响全拼 / 双拼的九条模糊音。
