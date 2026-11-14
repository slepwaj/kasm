"""把《常用字表_Xina切拼.xlsx》转成 KASM 的 Xina 切拼词库与音节表。

源表是「字 / 中古音 / 來源」三列，每个字一个读音；文本转简体，词频借用普通话词库。
"""

from pathlib import Path

import openpyxl
from opencc import OpenCC

ROOT = Path(r"D:\kasm")
SOURCE = ROOT / "tools/source/xina" / "常用字表_Xina切拼.xlsx"
if not SOURCE.is_file():
    SOURCE = ROOT / "常用字表_Xina切拼.xlsx"
TSV = ROOT / "assets/lexicon/xina-qiepin.tsv"
SYLLABLES = ROOT / "crates/kasm-core/data/xina-qiepin-syllables.txt"
DICT = ROOT / "assets/lexicon/dict.tsv"


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


def main():
    frequencies = read_frequencies()
    t2s = OpenCC("t2s")
    ws = openpyxl.load_workbook(SOURCE, read_only=True, data_only=True).worksheets[0]
    rows = {}
    syllables = set()
    for row in ws.iter_rows(min_row=2, values_only=True):
        if not row[0] or not row[1]:
            continue
        text = t2s.convert(str(row[0]).strip())
        reading = str(row[1]).strip()
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
