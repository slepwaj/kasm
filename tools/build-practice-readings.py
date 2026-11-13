"""给打字练习文段预生成逐字注音。

输出 `assets/practice/readings.tsv`：句子 + 各方案逐字读音（用 `|` 分隔）。
拼音用 pypinyin 出带调拼音；其他方案按词库最长词优先切分，词条读音音节数与字数一致时才按字拆开，
否则回落单字最高频读音。

依赖：pip install pypinyin
"""

from pathlib import Path

import re

from pypinyin import Style, lazy_pinyin

ROOT = Path(r"D:\kasm")
SENTENCES = ROOT / "assets/practice/sentences.txt"
OUT = ROOT / "assets/practice/readings.tsv"
SCHEMES = {
    "pinyin": "dict.tsv",
    "tungdzih": "tungdzih.tsv",
    "tupa": "tupa.tsv",
    "shanggu": "shanggu.tsv",
    "unt": "unt.tsv",
    "gcr": "gcr.tsv",
    "jyutping": "jyutping.tsv",
}


def load_dictionary(path: Path, strip_tones: bool = False):
    words: dict[str, tuple[str, int]] = {}
    chars: dict[str, tuple[str, int]] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 3:
            continue
        text, reading = fields[0], fields[1]
        if strip_tones:
            reading = re.sub(r"[0-5]", "", reading)
        try:
            frequency = int(fields[2])
        except ValueError:
            continue
        target = chars if len(text) == 1 else words
        previous = target.get(text)
        if previous is None or frequency > previous[1]:
            target[text] = (reading, frequency)
    return words, chars


def split_readings(reading: str) -> list[str]:
    return [part for part in reading.split() if part]


def annotate_pinyin(sentence: str) -> list[str]:
    readings = lazy_pinyin(sentence, style=Style.TONE)
    if len(readings) != len(sentence):
        return [""] * len(sentence)
    return [
        "" if not ("\u4e00" <= char <= "\u9fff") else reading
        for char, reading in zip(sentence, readings)
    ]


def annotate(sentence: str, words, chars) -> list[str]:
    result = [""] * len(sentence)
    index = 0
    while index < len(sentence):
        matched = False
        for size in range(min(8, len(sentence) - index), 1, -1):
            word = sentence[index : index + size]
            entry = words.get(word)
            if entry is None:
                continue
            syllables = split_readings(entry[0])
            if len(syllables) != size:
                continue
            result[index : index + size] = syllables
            index += size
            matched = True
            break
        if matched:
            continue
        entry = chars.get(sentence[index])
        result[index] = entry[0] if entry else ""
        index += 1
    return result


def main():
    sentences = [
        line.strip()
        for line in SENTENCES.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    ]
    dictionaries = {
        scheme: load_dictionary(
            ROOT / "assets/lexicon" / name,
            strip_tones=scheme == "pinyin",
        )
        for scheme, name in SCHEMES.items()
    }
    with OUT.open("w", encoding="utf-8", newline="\n") as output:
        output.write("# 练习文段逐字注音：句子+" + "".join(f"\t{scheme}" for scheme in SCHEMES) + "\n")
        output.write("# 读音之间用 | 分隔，空串表示词库里没有这个字的读音。\n")
        for sentence in sentences:
            output.write(sentence)
            for scheme, (words, chars) in dictionaries.items():
                readings = (
                    annotate_pinyin(sentence)
                    if scheme == "pinyin"
                    else annotate(sentence, words, chars)
                )
                output.write("\t" + "|".join(readings))
            output.write("\n")
    print("sentences", len(sentences), "wrote", OUT)


if __name__ == "__main__":
    main()
