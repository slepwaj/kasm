"""把 Rime 的吴语（苏州/混合腔）词典转成 KASM 词库与音节表。

Rime 方案把声调数字去掉后作为输入码（`xform/\\d//`），这里同样去掉末尾数字；
文本转简体，词频借用普通话词库。
"""

from pathlib import Path

from opencc import OpenCC

ROOT = Path(r"D:\kasm")
SOURCE = ROOT / "tools/source/wugniu" / "wugniu_suwu.dict.yaml"
if not SOURCE.is_file():
    SOURCE = ROOT / "rime-wugniu-suwu" / "wugniu_suwu.dict.yaml"
TSV = ROOT / "assets/lexicon/wugniu.tsv"
SYLLABLES = ROOT / "crates/kasm-core/data/wugniu-syllables.txt"
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


def bare(reading: str) -> str:
    out = []
    for syllable in reading.split():
        if syllable and syllable[-1].isdigit():
            syllable = syllable[:-1]
        out.append(syllable)
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
        if len(parts) < 2 or not parts[0].strip() or not parts[1].strip():
            continue
        text = t2s.convert(parts[0].strip())
        reading = bare(parts[1].strip())
        if not reading:
            continue
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
