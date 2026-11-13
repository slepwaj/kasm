"""从简体通字词库生成繁体词库与简繁对照表。

- 单字优先用《通字_合并.xlsx》里的繁体写法；
- 词条用 OpenCC 的 s2t 整词转换；
- 输出 `assets/lexicon/tungdzih-traditional.tsv` 与 `assets/lexicon/tungdzih-script.tsv`。

读音与词频原样保留；同一个 (繁体, 读音) 去重时取较大词频。

依赖：pip install openpyxl opencc-python-reimplemented
"""

import collections
from pathlib import Path

import openpyxl
from opencc import OpenCC

ROOT = Path(r"D:\kasm")
SOURCE = ROOT / "assets/lexicon/tungdzih.tsv"
XLSX = ROOT / "tools/source/tongzi.xlsx"
OUT = ROOT / "assets/lexicon/tungdzih-traditional.tsv"
SCRIPT = ROOT / "assets/lexicon/tungdzih-script.tsv"


def read_entries():
    rows = []
    for line in SOURCE.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        rows.append((parts[0], parts[1], int(parts[2])))
    return rows


def char_map():
    wb = openpyxl.load_workbook(XLSX, read_only=True, data_only=True)
    ws = wb["通字"] if "通字" in wb.sheetnames else wb.worksheets[0]
    t2s = OpenCC("t2s")
    s2t = OpenCC("s2t")
    forms = collections.defaultdict(list)
    for row in ws.iter_rows(min_row=2, values_only=True):
        raw, reading = row[0], row[1]
        if not raw or not reading:
            continue
        raw = str(raw).strip()
        simplified = t2s.convert(raw)
        if simplified != raw and raw not in forms[simplified]:
            forms[simplified].append(raw)
    mapping = {}
    for simplified, candidates in forms.items():
        default = s2t.convert(simplified)
        mapping[simplified] = default if default in candidates else candidates[0]
    return mapping, s2t


def main():
    entries = read_entries()
    mapping, s2t = char_map()
    seen = {}
    pairs = {}
    for text, reading, freq in entries:
        traditional = (
            mapping.get(text, s2t.convert(text))
            if len(text) == 1
            else s2t.convert(text)
        )
        if traditional != text:
            pairs.setdefault(traditional, text)
        key = (traditional, reading)
        if key not in seen or freq > seen[key]:
            seen[key] = freq

    with OUT.open("w", encoding="utf-8", newline="\n") as f:
        for (text, reading), freq in seen.items():
            f.write(f"{text}\t{reading}\t{freq}\n")
    with SCRIPT.open("w", encoding="utf-8", newline="\n") as f:
        f.write("# 通字简繁对照：繁体<TAB>简体；只列写法不同的条目。\n")
        for traditional, simplified in sorted(pairs.items()):
            f.write(f"{traditional}\t{simplified}\n")
    print("traditional entries", len(seen), "script pairs", len(pairs))
    print("wrote", OUT)
    print("wrote", SCRIPT)


if __name__ == "__main__":
    main()
