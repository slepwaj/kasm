# 输入法字词库

面向中英混合输入的分类文本字词库，包含规范汉字、现代汉语常用词、11类中文领域词和英文词库，提供TXT与TSV两种格式，可用于词条检索、候选生成和词库索引构建。

## 内容

| 目录 | 内容 |
| --- | --- |
| `01_characters` | 8,105个规范汉字；另分一级3,500字、二级3,000字、三级1,605字 |
| `02_common` | liuxilu版现代汉语常用词表；56,008个原始词号，拆分异写后56,064条记录、55,735个不同词形 |
| `03_domains` | IT与计算机、财经、地名、历史人物、医学、饮食、法律、汽车、动物、成语、诗词名句 |
| `04_internet_slang` | 网络用语扩展预留目录，无词条数据 |
| `05_english` | 英文补全总表114,876词形，含常用词、扩展词、专名与技术词；附词形变化及误拼对照 |
| `00_meta` | 分类统计、异写关系、异常DF、文件校验值与许可证 |

领域词跨类去重后共156,289个不同词形。各领域保留独立词表，允许同一词条属于多个领域。完整数量见[分类统计](00_meta/category_stats.tsv)。英文内容和字段见[英文词库说明](05_english/README.md)。

## 文件格式

所有文本采用UTF-8编码。以下统一字段适用于中文字词库；英文词表、词形变化和误拼对照使用各自的字段，详见英文目录。

- **TXT**：一行一个词形，同一文件内去重，适合词表查询或导入不需要权重的系统。
- **TSV**：Tab分隔、首行为字段名，适合保留读音、排序与来源的导入流程。

| 字段 | 含义 |
| --- | --- |
| `word` | 词条：汉字、词语或固定短语 |
| `pinyin` | 源词表提供的读音；仅通用词库有值 |
| `rank` | 排序号：标准字库中的字表编号，或通用词库中的原词号；不是出现次数 |
| `doc_freq` | THUOCL的DF，即包含该词的文档数 |
| `level` | 规范汉字的1、2、3级 |
| `source` | 数据来源标识 |

字段名、JSON 键与领域标识（`places`、`law`……）统一用英文，中文含义见本表与各文件说明。

空字段表示数据缺失，不表示零。通用词的排序号越小通常越常用。不同领域DF所依据的语料不同，不宜直接比较为统一权重。

## 使用

1. 选择规范字全表或分级字表，两者无需重复加载。
2. 导入通用词TSV；同一词形可能有不同读音或原词号，应按引擎规则合并。
3. 按需加载领域词库，保留多领域归属，并处理与通用词库的重叠。
4. 补充缺失读音、建立候选权重，再编译为输入法支持的查询索引。

通用词表的分号异写已拆为独立记录，主词形与异写的对应关系见[异写关系](00_meta/variant_forms.tsv)。领域词按词形去除首尾空白并去重，重复词形的有效DF取最大值；无效DF留空，原值见[异常DF](00_meta/anomalous_df.tsv)。

## sources

- **规范字**：[iDvel文字转录](https://github.com/iDvel/The-Table-of-General-Standard-Chinese-Characters)，以[shengdoushi分级字表](https://github.com/shengdoushi/common-standard-chinese-characters-table)交叉核对。两份字序一致。此数据仅提供字形与级别，不含完整读音。
- **通用词**：[liuxilu校对词表](https://github.com/liuxilu/Proofread-Modern-Chinese-Common-Lexicon)。读音仍需按输入法需求校验。
- **领域词**：[THUOCL](https://github.com/thunlp/THUOCL)，采用[MIT许可证](00_meta/THUOCL_LICENSE.txt)，分发时保留版权及许可声明。

英文来源为ESDB、CSpell和typos，许可声明见[英文sources](05_english/README.md#sources)。

本数据包不以单一许可证覆盖所有来源。上游文件、版本及质量参考见相邻的[资源归档](../输入法词库资源_2026-09-05/README.md)。


## 粤语词库（jyutping.tsv）

`jyutping.tsv` 是粤拼（无调 Jyutping）词库：134,285 条，格式 `词<TAB>音节（空格分隔）<TAB>词频`。
来源：CanCLID 的 Rime Cantonese 方案（https://github.com/CanCLID/rime-cantonese），按其数据许可 CC BY 4.0 / ODbL 使用；
音节表是其 729 个无调粤拼音节。转换脚本与许可原文见文言拼音项目 `data/licenses/`。

### 粤语 bigram（jyutping-unigram.tsv / jyutping-bigram.tsv）

`jyutping-unigram.tsv` 是词频（`词<TAB>词频`），`jyutping-bigram.tsv` 是词转移（`前词<TAB>后词<TAB>次数`）。
当前版本由 `jyutping.tsv` 的复合词按「左右两半都是词」拆出来计数，是**引导版**，只覆盖固定搭配；
换成按语料训练的结果时保持同样的两列/三列格式即可。

### 通字（tungdzih.tsv）

80,197 条（2026-11 按《通字_合并.xlsx》更新，前 55 条与「什」保持旧读）：来自 `baopaau/rime-tungdzih` 的 `tungdzih.dict.yaml`（整理自赵元任《通字方案》），
按KASM约定转成简体后入库；另用常用词表反向补了 52,508 个繁体→简体的通字词条（逐字取通字码），
词频借用普通话词表。源 README 注明「此码表大部分条目尚未校对，请勿作学习参考之用」。
仓库未声明许可证；再分发前请与原作者确认。

### 通字简繁两套

- `tungdzih.tsv`：简体词库（80,197 条）。
- `tungdzih-traditional.tsv`：标准繁体词库（80,038 条），单字优先取《通字_合并.xlsx》的繁体写法，词条用 OpenCC `s2t` 整词转换。
- `tungdzih-script.tsv`：繁体 → 简体对照（37,203 对），学习、日志按它还原成简体。

由 `tools/build-tungdzih-traditional.py` 从简体词库与《通字_合并.xlsx》生成；引擎按「输出字形」整份换词库，台湾 / 香港字形在繁体词库上再换。

单字排序用 `tools/source/tungdzih-priority.tsv`：由《通字（以ABC排序）.xlsx》转简体生成，表内字按行序排在生僻字前。

### Xina 拼音（xina.tsv）

来自用户提供的 Rime `xinapinin2`（作者 Xina，说明见 <https://zhuanlan.zhihu.com/p/2092749558438744805>），
`tools/build-xina.py` 把声调数字 1-5 转成 `!@#$%`、文本转简体后生成；音节表在
`crates/kasm-core/data/xina-syllables.txt`。词频借用普通话词库。

### Xina 切拼（xina-qiepin.tsv）

来自用户提供的 `常用字表_Xina切拼.xlsx`（字 / 中古音 / 來源），27k 条单字读音；
`tools/build-xina-qiepin.py` 转简体后生成，音节表在 `crates/kasm-core/data/xina-qiepin-syllables.txt`。

### 吳語（wugniu.tsv）

rime-wugniu-suwu（苏州/混合腔）的罗马字词库，`tools/build-wugniu.py` 去掉读音末尾的声调数字、
文本转简体后生成；音节表在 `crates/kasm-core/data/wugniu-syllables.txt`。

### 中古音罗马字（tupa.tsv）

来自 `D:\文言拼音\参考项目\古籍字頻統計_注音.xlsx` 的 tupa 列，23k 余条单字读音（含多音字），
字频按原表行序折算。

### 上古音（shanggu.tsv）

来自 `D:\文言拼音\参考项目\上古汉语.xlsx` 的「字典表」，11,091 条单字读音（含多音字）；
输入侧去掉 `C.` 前缀、把 `'` 改写成 `x` 后入库（`p'aq` → `pxaq`），字频借用普通话词表。
