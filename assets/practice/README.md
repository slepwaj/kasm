# 打字练习文段

Android 版「打字练习」随机抽句的来源。句子为本项目自写，随仓库以 GPL-3.0-or-later 发布。

- `sentences.txt`：练习句子源文件，一行一句，`#` 开头为注释。
- `readings.tsv`：由 `tools/build-practice-readings.py` 生成的简体 / 繁体文段与逐字注音（拼音 / 通字 / 中古音 / 上古音 / unt / 中文通用羅馬字 / 粤拼）；读音之间用 `|` 分隔。

练习页会随机抽几句拼成一段，所以每句都要能独立成句。改完句子后重跑：

```sh
pip install pypinyin
python tools/build-practice-readings.py
```
