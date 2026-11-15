"""把 TIPA 的 Type1 字体转成 Android 可用的 TTF。

TIPA 的 PFB 用 T3 编码，槽位和 Unicode 不是一一对应；`t3enc.def` 给出
槽位 → TIPA 命令，`tipauni.dtx` 给出命令 → Unicode。这里只保留能映射的槽位。
"""

import re
from pathlib import Path

from fontTools import t1Lib
from fontTools.fontBuilder import FontBuilder
from fontTools.pens.cu2quPen import Cu2QuPen
from fontTools.pens.ttGlyphPen import TTGlyphPen

ROOT = Path(r"D:\kasm")
TIPA = ROOT / "tipa"
PFB = TIPA / "type1/tipa10.pfb"
PFA = ROOT / "target/tipa10.pfa"
T3ENC = TIPA / "sty/t3enc.def"
TIPAUNI = ROOT / "tools/source/tipauni.dtx"
OUT = ROOT / "apps/android/app/src/main/res/font/tipa.ttf"

def read_t3():
    slots = {}
    for line in T3ENC.read_text(encoding="latin-1").splitlines():
        match = re.search(r"DeclareTextSymbol\\(\w+)\{T3\}\{(\d+)\}", line)
        if match:
            slots[int(match.group(2))] = match.group(1)
    return slots


def read_unicode():
    mapping = {}
    for line in TIPAUNI.read_text(encoding="utf-8", errors="replace").splitlines():
        match = re.search(r"DeclareTextSymbol\{\\(\w+)\}\{TU\}\{\"([0-9A-Fa-f]+)\}", line)
        if match:
            mapping[match.group(1)] = chr(int(match.group(2), 16))
    return mapping


def read_encoding():
    text = PFA.read_text(encoding="latin-1")
    return {int(code): name for code, name in re.findall(r"dup (\d+) /(\w+) put", text)}


def prepare_pfa():
    if not PFA.exists():
        PFA.parent.mkdir(parents=True, exist_ok=True)
        t1Lib.T1Font(PFB).saveAs(PFA, "pfa")


NAME_TO_UNICODE = {
    "agrave": "à", "aacute": "á", "aelig": "æ", "ae": "æ", "ccedilla": "ç", "eth": "ð",
    "oslash": "ø", "eogonek": "ę", "ncedilla": "ņ", "eng": "ŋ", "oe": "œ",
    "scedilla": "ş", "tcedilla": "ţ", "yogh": "ȝ", "wynn": "ꝩ",
    "middot": "·", "bar": "ǀ", "exclam": "ǃ", "brokenbar": "‖", "undertie": "‿",
}

EXTRA_SLOT = {
    78: "ŋ", 86: "ʋ", 68: "ð", 196: "ɚ", 198: "ɞ", 199: "ɝ", 201: "ʛ", 213: "ɺ",
    235: "ɫ", 243: "ɻ", 249: "ʂ", 250: "ʈ", 252: "ʐ", 253: "ʑ", 215: "ɶ",
    34: "ˈ", 35: "ˌ", 58: "ː", 59: "ˑ", 124: "‖",
}


def main():
    prepare_pfa()
    t3 = read_t3()
    unicode = read_unicode()
    encoding = read_encoding()
    font = t1Lib.T1Font(PFA)
    glyph_set = font.getGlyphSet()

    glyphs = {".notdef": TTGlyphPen(None).glyph()}
    order = [".notdef"]
    cmap = {}
    metrics = {".notdef": (1000, 0)}
    for slot in range(256):
        name = encoding.get(slot)
        command = t3.get(slot)
        char = unicode.get(command) if command else None
        if char is None:
            char = EXTRA_SLOT.get(slot)
        if char is None and command is None and name in NAME_TO_UNICODE:
            char = NAME_TO_UNICODE[name]
        # 普通 ASCII 槽位（T3 特殊槽位不覆盖）：TIPA 字体里的 a / p / t 等仍是拉丁字形
        if char is None and command is None and name and len(name) == 1 and 32 <= slot <= 126:
            char = chr(slot)
        if not name or name == ".notdef" or char is None:
            continue
        # 已经收过同一个字符时不再覆盖；TIPA 有些槽位是变体
        codepoint = ord(char)
        if codepoint in cmap:
            continue
        if name in glyphs:
            cmap[codepoint] = name
            continue
        tt_pen = TTGlyphPen(None)
        pen = Cu2QuPen(tt_pen, 1.0)
        glyph_set[name].draw(pen)
        glyphs[name] = tt_pen.glyph()
        order.append(name)
        cmap[codepoint] = name
        metrics[name] = (int(glyph_set[name].width), 0)

    fb = FontBuilder(1000, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap(cmap)
    fb.setupGlyf(glyphs)
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=800, descent=-200)
    fb.setupOS2(sTypoAscender=800, sTypoDescender=-200, usWinAscent=800, usWinDescent=200)
    fb.setupNameTable({"familyName": "TIPA", "styleName": "Regular", "fullName": "TIPA Regular"})
    fb.setupPost()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    fb.save(OUT)
    print("glyphs", len(order), "cmap", len(cmap), "wrote", OUT)


if __name__ == "__main__":
    main()
