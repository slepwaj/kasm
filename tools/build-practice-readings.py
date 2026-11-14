"""给打字练习文段预生成逐字注音。

输出 `assets/practice/readings.tsv`：句子 + 各方案逐字读音（用 `|` 分隔）。
拼音用 pypinyin 出带调拼音；其他方案按词库最长词优先切分，词条读音音节数与字数一致时才按字拆开，
否则回落单字最高频读音。

依赖：pip install pypinyin
"""

from pathlib import Path

import importlib.util
import re

from opencc import OpenCC
from pypinyin import Style, lazy_pinyin

ROOT = Path(r"D:\kasm")
SENTENCES = ROOT / "assets/practice/sentences.txt"
CORPORA = {
    "modern": SENTENCES,
    "cantonese": ROOT / "assets/practice/sentences-cantonese.txt",
    "classical": ROOT / "assets/practice/sentences-classical.txt",
    "wugniu": ROOT / "assets/practice/sentences-wugniu.txt",
    "tungdzih": ROOT / "tools/source/practice/tungdzih-annotated.txt",
}
OUT = ROOT / "assets/practice/readings.tsv"
TONES = ROOT / "crates/kasm-core/data/tones.tsv"
SCHEMES = {
    "pinyin": "dict.tsv",
    "tungdzih": "tungdzih.tsv",
    "tupa": "tupa.tsv",
    "shanggu": "shanggu.tsv",
    "unt": "unt.tsv",
    "gcr": "gcr.tsv",
    "jyutping": "jyutping.tsv",
    "xina": "xina.tsv",
    "xina-qiepin": "xina-qiepin.tsv",
    "wugniu": "wugniu.tsv",
}


def load_xina_converter():
    spec = importlib.util.spec_from_file_location("build_xina", ROOT / "tools/build-xina.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.xina


def load_tones():
    table = {}
    for line in TONES.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 2:
            continue
        readings = []
        for item in parts[1].split():
            if item and item[-1].isdigit():
                readings.append((item[:-1], int(item[-1])))
        table[parts[0][0]] = readings
    return table


XINA = load_xina_converter()
TONES = load_tones()
XINA_TONE = {}
for _char, _readings in TONES.items():
    forms = {}
    for _syllable, _tone in _readings:
        forms.setdefault(XINA(_syllable), _tone)
    XINA_TONE[_char] = forms

TONE_MARKS = {
    "a": ("a", "á", "ǎ", "à", "ạ"),
    "e": ("e", "é", "ě", "è", "ẹ"),
    "i": ("i", "í", "ǐ", "ì", "ị"),
    "o": ("o", "ó", "ǒ", "ò", "ọ"),
    "u": ("u", "ú", "ǔ", "ù", "ụ"),
}


def mark_xina(reading: str, char: str) -> str:
    tone = XINA_TONE.get(char, {}).get(reading)
    if not tone:
        return reading
    for index, character in enumerate(reading):
        if character in TONE_MARKS:
            marked = TONE_MARKS[character][tone - 1]
            return reading[:index] + marked + reading[index + 1 :]
    return reading

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


def parse_annotated(path: Path):
    """解析「字(通字读音)」形式的文段，返回 (去掉注音的句子, 逐字读音)。"""
    text = path.read_text(encoding="utf-8-sig")
    pattern = re.compile(r"([\u4e00-\u9fff])\(([^)]+)\)")
    clean: list[str] = []
    readings: list[str] = []
    position = 0
    for match in pattern.finditer(text):
        for ch in text[position : match.start()]:
            if ch not in "\r\n":
                clean.append(ch)
                readings.append("")
        clean.append(match.group(1))
        readings.append(match.group(2))
        position = match.end()
    for ch in text[position:]:
        if ch not in "\r\n":
            clean.append(ch)
            readings.append("")
    sentences = []
    current: list[str] = []
    current_readings: list[str] = []
    for ch, reading in zip(clean, readings):
        current.append(ch)
        current_readings.append(reading)
        if ch in "。！？":
            sentences.append(("".join(current), current_readings))
            current = []
            current_readings = []
    if current:
        sentences.append(("".join(current), current_readings))
    for sentence, values in sentences:
        for ch, reading in zip(sentence, values):
            if "\u4e00" <= ch <= "\u9fff" and not reading:
                raise ValueError(f"缺少通字注音：{ch} in {sentence}")
    return sentences


def main():
    corpora = {}
    for name, path in CORPORA.items():
        if name == "tungdzih":
            corpora[name] = parse_annotated(path)
        else:
            corpora[name] = [
                (line.strip(), None)
                for line in path.read_text(encoding="utf-8").splitlines()
                if line.strip() and not line.startswith("#")
            ]
    dictionaries = {
        scheme: load_dictionary(
            ROOT / "assets/lexicon" / name,
            strip_tones=scheme == "pinyin",
        )
        for scheme, name in SCHEMES.items()
    }
    s2t = OpenCC("s2t")
    t2s = OpenCC("t2s")
    with OUT.open("w", encoding="utf-8", newline="\n") as output:
        output.write("# 练习文段逐字注音：语料\t简体\t繁体 + " + "".join(f"\t{scheme}" for scheme in SCHEMES) + "\n")
        output.write("# 语料：modern / cantonese / classical / wugniu / tungdzih；读音之间用 | 分隔，空串表示词库里没有这个字的读音。\n")
        for corpus, sentences in corpora.items():
            for source, overrides in sentences:
                simplified = t2s.convert(source)
                traditional = s2t.convert(simplified)
                if len(simplified) != len(traditional):
                    raise ValueError(f"简繁转换改变了字数：{source} -> {traditional}")
                output.write(corpus)
                output.write("\t" + simplified)
                output.write("\t" + traditional)
                for scheme, (words, chars) in dictionaries.items():
                    # 粤拼词库收的是传统字形（佢、哋、嘅），按原文注；其余方案按简体注。
                    annotation = source if scheme == "jyutping" else simplified
                    if scheme == "tungdzih" and overrides is not None:
                        readings = overrides
                    else:
                        readings = (
                            annotate_pinyin(annotation)
                            if scheme == "pinyin"
                            else annotate(annotation, words, chars)
                        )
                    if len(readings) != len(simplified):
                        raise ValueError(f"注音长度不符：{scheme} {source}")
                    if scheme == "xina":
                        readings = [
                            mark_xina(reading, char)
                            for reading, char in zip(readings, simplified)
                        ]
                    output.write("\t" + "|".join(readings))
                output.write("\n")
    print("corpora", {name: len(lines) for name, lines in corpora.items()}, "wrote", OUT)


if __name__ == "__main__":
    main()
