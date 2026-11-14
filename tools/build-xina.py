"""把普通话词库的拼音转成《Xina传统派普通话拼音方案》。

源规范：`tools/source/xina/Xina拼音.docx`（声母、韵母、声调表）。
输入词库不带声调，这里只生成无调 Xina 拼式；声调仍可按文档写成符号或数字。
"""

from pathlib import Path

ROOT = Path(r"D:\kasm")
DICT = ROOT / "assets/lexicon/dict.tsv"
OUT = ROOT / "assets/lexicon/xina.tsv"
SYLLABLES = ROOT / "crates/kasm-core/data/xina-syllables.txt"
TONES = ROOT / "crates/kasm-core/data/tones.tsv"
TONE_MARKS = {"1": "!", "2": "@", "3": "#", "4": "$", "5": "%"}

INITIALS = (
    "zh", "ch", "sh",
    "b", "p", "m", "f", "d", "t", "n", "l", "g", "k", "h",
    "j", "q", "x", "r", "z", "c", "s",
)

FINALS = {
    "a": "a", "o": "o", "e": "e", "ai": "ai", "ei": "ei",
    "ao": "au", "ou": "ou", "an": "an", "en": "en",
    "ang": "ang", "eng": "eng", "er": "eh",
    "i": "i", "ia": "ya", "ie": "ye", "iao": "yau", "iu": "you",
    "ian": "yan", "in": "in", "iang": "yang", "ing": "ing", "iong": "yung",
    "u": "u", "ua": "wa", "uo": "wo", "uai": "wai", "ui": "wei",
    "uan": "wan", "un": "wen", "uang": "wang", "ong": "ung",
    "v": "yu", "ve": "ywe", "van": "ywan", "vn": "yun",
}

ZERO = {
    "yi": "i", "ya": "ya", "ye": "ye", "yao": "yau", "you": "you",
    "yan": "yan", "yin": "in", "yang": "yang", "ying": "ing",
    "yong": "yung", "yu": "yu", "yue": "ywe", "yuan": "ywan", "yun": "yun",
    "wu": "u", "wa": "wa", "wo": "wo", "wai": "wai", "wei": "wei",
    "wan": "wan", "wen": "wen", "wang": "wang", "weng": "weng",
    "er": "eh",
}


def strip_tone(syllable: str) -> str:
    return syllable[:-1] if syllable and syllable[-1].isdigit() else syllable


def xina(syllable: str) -> str:
    syllable = strip_tone(syllable).lower()
    if syllable in ZERO:
        return ZERO[syllable]
    if syllable in FINALS:
        return FINALS[syllable]

    initial = next((item for item in INITIALS if syllable.startswith(item)), "")
    if not initial:
        return syllable

    final = syllable[len(initial):]
    # j/q/x 后的 u 是 ü：ju → jyu，jue → jywe，juan → jywan，jun → jyun
    if initial in ("j", "q", "x"):
        if final in ("u", "ue", "uan", "un"):
            final = {"u": "v", "ue": "ve", "uan": "van", "un": "vn"}[final]
        if final == "iong":
            final = "iong"
    # 舌尖元音：zhi/chi/shi/ri/zi/ci/si → ih
    if final == "i" and initial in ("zh", "ch", "sh", "r", "z", "c", "s"):
        return initial + "ih"
    return initial + FINALS.get(final, final)


def read_tones():
    table = {}
    for line in TONES.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 2 or not parts[0]:
            continue
        ch = parts[0][0]
        readings = []
        for item in parts[1].split():
            if item and item[-1].isdigit():
                readings.append((item[:-1], int(item[-1])))
        table[ch] = readings
    return table


def tone_forms(text: str, pinyin, tones, base: str):
    """给单字或词生成带调拼式；多字词取每个字最常用的同音声调。"""
    syllables = pinyin.split()
    chars = list(text)
    if len(chars) != len(syllables):
        return []
    results = []
    choices = []
    for ch, syllable in zip(chars, syllables):
        readings = tones.get(ch, [])
        matching = [tone for reading, tone in readings if reading == strip_tone(syllable)]
        if matching:
            choices.append(matching)
        else:
            choices.append([1])
    if len(chars) == 1:
        # 单字把同音的所有声调都收进来
        for tone in choices[0]:
            results.append(base + TONE_MARKS[str(tone)])
    else:
        # 词只生成一组：每个字取第一个可用声调
        parts = []
        for xina_syllable, tones_for_char in zip(base.split(), choices):
            parts.append(xina_syllable + TONE_MARKS[str(tones_for_char[0])])
        results.append(" ".join(parts))
    return results


def main():
    rows = {}
    syllables = set()
    tones = read_tones()
    for line in DICT.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) < 3:
            continue
        text = parts[0]
        pinyin = parts[1]
        reading = " ".join(xina(syllable) for syllable in pinyin.split())
        try:
            frequency = int(parts[2])
        except ValueError:
            frequency = 1
        key = (text, reading)
        rows[key] = max(rows.get(key, 0), max(1, frequency))
        for form in tone_forms(text, pinyin, tones, reading):
            syllables.update(form.split())
            tone_key = (text, form)
            rows[tone_key] = max(rows.get(tone_key, 0), max(1, frequency // 2))

    with OUT.open("w", encoding="utf-8", newline="\n") as output:
        for (text, reading), frequency in rows.items():
            output.write(f"{text}\t{reading}\t{frequency}\n")
    syllables.update(s for _, reading in rows for s in reading.split())
    with SYLLABLES.open("w", encoding="utf-8", newline="\n") as output:
        output.write("\n".join(sorted(syllables)) + "\n")
    print("entries", len(rows), "syllables", len(syllables))
    print("wrote", OUT)
    print("wrote", SYLLABLES)


if __name__ == "__main__":
    main()
