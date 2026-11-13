#!/usr/bin/env python3
"""用《通字.xlsx》重建通字词库与音节表。

默认读 `tools/source/tongzi.xlsx`（2026-11 起是《通字_合并.xlsx》的副本）。
`tools/source/tungdzih-keep-old.tsv` 里的字强制保留旧读（人工确认不做改动的例外）。
- 单字：以 xlsx 为准（繁体先转简体），旧表里 xlsx 没有的生僻字保持原样；
- 词：按字拆分读音，某个字的旧读被删掉且新读只剩一个时替换；
  新读不止一个（多音字）时保留旧词读音，避免改错；
- 音节表：从最终词库重新收集，排序去重。

依赖：pip install openpyxl opencc-python-reimplemented
用法：python tools/rebuild-tungdzih.py [通字.xlsx]
"""

import collections
import sys
from pathlib import Path

import openpyxl
from opencc import OpenCC

ROOT = Path(__file__).resolve().parent.parent
XLSX = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "tools/source/tongzi.xlsx"
TSV = ROOT / "assets/lexicon/tungdzih.tsv"
SYLLABLES = ROOT / "crates/kasm-core/data/tungdzih-syllables.txt"
KEEP_OLD = ROOT / "tools/source/tungdzih-keep-old.tsv"


def read_keep_old():
    """人工确认保持旧读的字：字<TAB>旧读音（空格分隔）。"""
    if not KEEP_OLD.is_file():
        return {}
    readings = {}
    for line in KEEP_OLD.read_text(encoding="utf-8-sig").splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        readings[parts[0]] = parts[1].split()
    return readings


def read_xlsx():
    wb = openpyxl.load_workbook(XLSX, read_only=True, data_only=True)
    # 旧表是 Sheet1；2026-11 的《通字_合并.xlsx》把数据放在「通字」页。
    for name in ("通字", "Sheet1"):
        if name in wb.sheetnames:
            ws = wb[name]
            break
    else:
        ws = wb.worksheets[0]
    cc = OpenCC("t2s")
    readings = collections.OrderedDict()
    for row in ws.iter_rows(min_row=2, values_only=True):
        hanzi, reading = row[0], row[1]
        if not hanzi or not reading:
            continue
        hanzi = cc.convert(str(hanzi).strip())
        reading = str(reading).strip()
        bucket = readings.setdefault(hanzi, [])
        if reading not in bucket:
            bucket.append(reading)
    for hanzi, old in read_keep_old().items():
        if hanzi in readings:
            readings[hanzi] = old
    return readings


def read_tsv():
    singles = collections.OrderedDict()
    words = []
    for line in TSV.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        text, reading, freq = parts[0], parts[1], int(parts[2])
        if len(text) == 1:
            singles.setdefault(text, []).append((reading, freq))
        else:
            words.append((text, reading, freq))
    return singles, words


def rebuild():
    xlsx = read_xlsx()
    singles, words = read_tsv()

    # 单字：xlsx 覆盖，保留 xlsx 未收录的生僻字
    rebuilt_singles = collections.OrderedDict()
    for hanzi, old in singles.items():
        if hanzi in xlsx:
            freq = max(freq for _, freq in old)
            rebuilt_singles[hanzi] = [(reading, freq) for reading in xlsx[hanzi]]
        else:
            rebuilt_singles[hanzi] = old
    for hanzi, readings in xlsx.items():
        if hanzi not in rebuilt_singles:
            rebuilt_singles[hanzi] = [(reading, 100) for reading in readings]

    old_readings = {hanzi: {r for r, _ in rows} for hanzi, rows in singles.items()}

    replaced = kept = ambiguous = 0
    rebuilt_words = []
    for text, reading, freq in words:
        syllables = reading.split()
        if len(text) != len(syllables):
            rebuilt_words.append((text, reading, freq))
            continue
        out = []
        word_ambiguous = False
        word_replaced = False
        for hanzi, syllable in zip(text, syllables):
            new = xlsx.get(hanzi)
            old = old_readings.get(hanzi, set())
            if not new or old == set(new):
                out.append(syllable)
                continue
            if syllable in new:
                kept += 1
                out.append(syllable)
            elif len(new) == 1:
                replaced += 1
                word_replaced = True
                out.append(new[0])
            else:
                ambiguous += 1
                word_ambiguous = True
                out.append(syllable)
        if word_ambiguous:
            rebuilt_words.append((text, reading, freq))
            continue
        rebuilt_words.append((text, " ".join(out), freq))

    with TSV.open("w", encoding="utf-8", newline="\n") as f:
        for hanzi, rows in rebuilt_singles.items():
            for reading, freq in rows:
                f.write(f"{hanzi}\t{reading}\t{freq}\n")
        for text, reading, freq in rebuilt_words:
            f.write(f"{text}\t{reading}\t{freq}\n")

    syllables = set()
    for rows in rebuilt_singles.values():
        syllables.update(reading for reading, _ in rows)
    for _, reading, _ in rebuilt_words:
        syllables.update(reading.split())
    with SYLLABLES.open("w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(sorted(syllables)) + "\n")

    print(
        f"单字 {len(rebuilt_singles)}，词 {len(rebuilt_words)}，"
        f"音节 {len(syllables)}；词内替换 {replaced} 个音节，"
        f"歧义保留 {ambiguous} 个"
    )


if __name__ == "__main__":
    rebuild()
