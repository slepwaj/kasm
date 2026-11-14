"""把 Rime 的 Xina 拼音词典转成 KASM 词库与音节表。

Xina 的声调用数字 1-5 表示；KASM 的罗马字音节表把声调写成 !@#$%（与 Rime schema 一致），
所以每个音节的末尾数字转成对应符号。词条文本先转简体，词频借用现有普通话词库。
"""

import collections
from pathlib import Path

from opencc import OpenCC

ROOT = Path(r"D:\kasm")
SOURCE = ROOT / "tools/source/xina" / "xinapinin2.dict.yaml"
if not SOURCE.is_file():
    SOURCE = ROOT / "Xina拼音输入法（带声调版）" / "xinapinin2.dict.yaml"
TSV = ROOT / "assets/lexicon/xina.tsv"
SYLLABLES = ROOT / "crates/kasm-core/data/xina-syllables.txt"
DICT = ROOT / "assets/lexicon/dict.tsv"
TONES = {"1": "!", "2": "@", "3": "#", "4": "$", "5": "%"}


def read_frequencies():
    frequencies = {}
    for line in DICT.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        try:
            frequency = int(parts[2])
        except ValueError:
            continue
        frequencies[parts[0]] = max(frequencies.get(parts[0], 0), frequency)
    return frequencies


def toned(reading: str) -> str:
    out = []
    for syllable in reading.split():
        if not syllable or syllable[-1] not in TONES:
            raise ValueError(f"missing tone: {syllable!r}")
        out.append(syllable[:-1] + TONES[syllable[-1]])
    return " ".join(out)


def main():
    frequencies = read_frequencies()
    t2s = OpenCC("t2s")
    rows = {}
    syllables = set()
    for line in SOURCE.read_text(encoding="utf-8").splitlines():
        if not line.strip() or line.startswith("#") or line.startswith("---") or line.startswith("..."):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        text = t2s.convert(parts[0])
        reading = toned(parts[1])
        syllables.update(reading.split())
        key = (text, reading)
        rows[key] = max(rows.get(key, 0), max(1, frequencies.get(text, 1)))

    with TSV.open("w", encoding="utf-8", newline="\n") as output:
        for (text, reading), frequency in rows.items():
            output.write(f"{text}\t{reading}\t{frequency}\n")
    with SYLLABLES.open("w", encoding="utf-8", newline="\n") as output:
        output.write("\n".join(sorted(syllables)) + "\n")
    print("entries", len(rows), "syllables", len(syllables))
    print("wrote", TSV)
    print("wrote", SYLLABLES)


if __name__ == "__main__":
    main()
