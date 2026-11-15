package app.kasm.inputmethod.keyboard.layout

import app.kasm.inputmethod.core.store.ShuangpinKeycap

/** 键位表：每行的键按 weight 分行宽，不满 10 个单位的行两边留白居中。 */
object KeyboardLayout {
    const val UNITS_PER_ROW = 10f

    /** 九宫格左侧拼音列的宽度；右列功能键是 1.5，中间三个数字键平分剩下的。 */
    const val SIDE_WIDTH = 1.5f
    const val NINE_KEY_WIDTH = (UNITS_PER_ROW - SIDE_WIDTH - 1.5f) / 3

    /** 九宫格里拼音列占的行数（前三行），这几行的键从拼音列右边排起。 */
    const val SIDE_ROWS = 3

    /** 中文标点键：全角的样子与对应的半角字符；半角标点设置下键上印半角。 */
    private val PUNCTUATION = mapOf('，' to ',', '。' to '.', '？' to '?', '！' to '!', '：' to ':')

    /** 中文键盘的一个标点键（[full] 是全角的样子），见 [Key.Text.punct]。 */
    fun punct(full: Char, fullWidth: Boolean, weight: Float = 1f): Key.Text {
        val half = PUNCTUATION.getValue(full)
        return Key.Text(if (fullWidth) full.toString() else half.toString(), weight, punct = half)
    }

    /** 不在组句时拼音列里放的标点；、和 … 没有对应的半角键，总是全角。 */
    fun sidePunctuation(fullWidth: Boolean): List<String> =
        listOf('，', '。', '？', '！').map { punct(it, fullWidth).text } + listOf("、", punct('：', fullWidth).text, "…")

    /** 拼音列里点的标点按哪个键算：认得的走 Core 的标点转换，其余原样。 */
    fun sideKey(item: String): Key.Text {
        val full = PUNCTUATION.entries.firstOrNull { (full, half) -> item == full.toString() || item == half.toString() }?.key
        return Key.Text(item, punct = full?.let(PUNCTUATION::getValue))
    }

    /** 字母键角标：首排数字，二排常用符号（中文把 H J 换成中文引号），三排括号与标点。 */
    private const val TOP_HINTS = "1234567890"
    private const val MIDDLE_HINTS_EN = "~!@#%'&*?"
    private const val MIDDLE_HINTS_ZH = "~!@#%“”*?"
    private const val BOTTOM_HINTS = "()-_:;/"

    /** 日语罗马音的角标：二排换成 ¥ 与日文括号「」，三排是引号与句读。 */
    private const val MIDDLE_HINTS_JA = "@#¥_&「」()"
    private const val BOTTOM_HINTS_JA = "*\"':;!?"

    private fun letters(row: String, hints: String) = row.zip(hints) { char, hint -> Key.Letter(char, hint.toString()) }

    /** 底排：两个切换键、逗号、空格、句号、中英、回车。中文的逗号句号走 Core 的标点转换（[fullWidth] 只管键上印什么）。 */
    private fun bottomRow(first: Key, second: Key, chinese: Boolean, fullWidth: Boolean = true) = listOf(
        first,
        second,
        if (chinese) punct('，', fullWidth) else Key.Text(","),
        Key.Space(),
        if (chinese) punct('。', fullWidth) else Key.Text("."),
        Key.Language,
        Key.Enter(),
    )

    /** 中文底排：「符」进分类符号面板；英文底排：「!?#」进常用符号页；「123」都进数字键盘。 */
    private fun lettersBottom(chinese: Boolean, fullWidth: Boolean = true) = if (chinese) {
        bottomRow(Key.Symbols("符", 1.25f), Key.NumberPage("123"), chinese = true, fullWidth)
    } else {
        bottomRow(Key.SymbolPage("!?#", 1.25f), Key.NumberPage("123"), chinese = false)
    }

    fun qwerty(chinese: Boolean, japanese: Boolean = false, fullWidth: Boolean = true): List<List<Key>> = if (japanese) {
        japaneseRows()
    } else {
        listOf(
            letters("qwertyuiop", TOP_HINTS),
            letters("asdfghjkl", if (chinese) MIDDLE_HINTS_ZH else MIDDLE_HINTS_EN),
            listOf(Key.Shift) + letters("zxcvbnm", BOTTOM_HINTS) + Key.Backspace,
            lettersBottom(chinese, fullWidth),
        )
    }

    /**
     * Xina 拼音：全键盘上方多一排声调键，键面写数字与调名，按下传 `!@#$%`。
     */
    fun xina(chinese: Boolean, fullWidth: Boolean = true): List<List<Key>> {
        if (!chinese) return qwerty(false, fullWidth = fullWidth)
        val tones = listOf(
            Key.Text("!", 2f, label = "1 阴平"),
            Key.Text("@", 2f, label = "2 阳平"),
            Key.Text("#", 2f, label = "3 上声"),
            Key.Text("$", 2f, label = "4 去声"),
            Key.Text("%", 2f, label = "5 轻声"),
        )
        return listOf(
            tones,
            letters("qwertyuiop", TOP_HINTS),
            letters("asdfghjkl", MIDDLE_HINTS_ZH),
            listOf(Key.Shift) + letters("zxcvbnm", BOTTOM_HINTS) + Key.Backspace,
            lettersBottom(chinese, fullWidth),
        )
    }

    /**
     * 注音大千：四排注音照电脑大千键位排，首排 11 个键（含声调）略窄，
     * 末排右边是删除；底排与拼音全键盘一样。空格是一声（还缺声调时）。
     */
    fun zhuyin(fullWidth: Boolean = true): List<List<Key>> {
        fun row(codes: String, symbols: String, weight: Float) =
            codes.zip(symbols) { code, symbol -> Key.Zhuyin(code, symbol.toString(), weight) }
        val (top, second, third, bottom) = ZHUYIN_ROWS
        return listOf(
            row(top.first, top.second, UNITS_PER_ROW / top.first.length),
            row(second.first, second.second, 1f),
            row(third.first, third.second, 1f),
            row(bottom.first, bottom.second, ZHUYIN_BOTTOM_WIDTH) + Key.Backspace,
            lettersBottom(chinese = true, fullWidth),
        )
    }

    private val ZHUYIN_ROWS = listOf(
        "1234567890-" to "ㄅㄉˇˋㄓˊ˙ㄚㄞㄢㄦ",
        "qwertyuiop" to "ㄆㄊㄍㄐㄔㄗㄧㄛㄟㄣ",
        "asdfghjkl;" to "ㄇㄋㄎㄑㄕㄘㄨㄜㄠㄤ",
        "zxcvbnm,./" to "ㄈㄌㄏㄒㄖㄙㄩㄝㄡㄥ",
    )

    private const val ZHUYIN_BOTTOM_WIDTH = 0.85f

    /**
     * 日语罗马音：二排末尾补一个长音「ー」（敲罗马音的 `-`）占满十格；
     * 底排与拼音同宽，标点是「、」「。」。
     */
    private fun japaneseRows() = listOf(
        letters("qwertyuiop", TOP_HINTS),
        letters("asdfghjkl", MIDDLE_HINTS_JA) + Key.Text("ー"),
        listOf(Key.Shift) + letters("zxcvbnm", BOTTOM_HINTS_JA) + Key.Backspace,
        listOf(Key.SymbolPage("!?#", 1.25f), Key.NumberPage("123"), Key.Text("、"), Key.Space(), Key.Text("。"), Key.Language, Key.Enter()),
    )

    /**
     * 双拼：键位同拼音全键盘，键上另印当前方案的声韵母（从 Core 取，见 [Shuangpin][app.kasm.inputmethod.core.store.Shuangpin]）；
     * 用 `;` 当韵母的方案二排末尾补一个 `;` 键；微软双拼照手机上的习惯（搜狗等）把 `;` 放在 Shift 的位置，
     * 点按永远是 `;`、上滑是 Shift（不按组句状态换含义），二排回到九个键。[hints] 关了不印声韵母，键面同拼音全键盘。
     */
    fun shuangpin(
        keycaps: Map<Char, ShuangpinKeycap>,
        scheme: String,
        fullWidth: Boolean = true,
        hints: Boolean = true,
    ): List<List<Key>> {
        val semicolonShift = scheme == MICROSOFT_SHUANGPIN
        fun letter(char: Char, hint: String, weight: Float = 1f, flickShift: Boolean = false): Key.Letter {
            val keycap = keycaps[char]?.takeIf { hints }
            return Key.Letter(char, hint, keycap?.initial.orEmpty(), keycap?.finals.orEmpty(), weight, flickShift)
        }
        fun row(chars: String, hints: String) = chars.mapIndexed { i, char -> letter(char, hints.getOrNull(i)?.toString().orEmpty()) }
        val semicolon = ';' in keycaps
        val shift = if (semicolon && semicolonShift) letter(';', "", weight = 1.5f, flickShift = true) else Key.Shift
        val middle = if (semicolon && !semicolonShift) "asdfghjkl;" else "asdfghjkl"
        return listOf(
            row("qwertyuiop", TOP_HINTS),
            row(middle, MIDDLE_HINTS_ZH),
            listOf(shift) + row("zxcvbnm", BOTTOM_HINTS) + Key.Backspace,
            lettersBottom(chinese = true, fullWidth),
        )
    }

    /** 微软双拼在 Core 方案表里的名字（`Scheme::key`）。 */
    private const val MICROSOFT_SHUANGPIN = "microsoft"

    fun nineKey(fullWidth: Boolean = true): List<List<Key>> = listOf(
        listOf(Key.Digit('1', "分词"), Key.Digit('2', "ABC"), Key.Digit('3', "DEF"), Key.Backspace),
        listOf(Key.Digit('4', "GHI"), Key.Digit('5', "JKL"), Key.Digit('6', "MNO"), Key.Clear),
        listOf(Key.Digit('7', "PQRS"), Key.Digit('8', "TUV"), Key.Digit('9', "WXYZ"), Key.Text("0", 1.5f)),
        lettersBottom(chinese = true, fullWidth),
    )

    /** 数字键盘左列的符号，可以上下拖（与九宫格拼音列同一个控件）。 */
    val NUMBER_SIDE = listOf("%", "/", "-", "+", "=", "*", ",", ":", "(", ")", "#", "\$")

    /**
     * 数字键盘：左列符号，中间 3×3 数字，右列 删除 / . / @；底排 符 / 返回 / 0 / 空格 / 回车，
     * 与上面各列对齐，所以复用九宫格的列宽。
     */
    fun numbers(): List<List<Key>> {
        fun digits(row: String) = row.map { Key.Text(it.toString(), NINE_KEY_WIDTH) }
        return listOf(
            digits("123") + Key.Backspace,
            digits("456") + Key.Text(".", SIDE_WIDTH),
            digits("789") + Key.Text("@", SIDE_WIDTH),
            listOf(
                Key.Symbols("符", SIDE_WIDTH),
                Key.Letters("返回", NINE_KEY_WIDTH),
                Key.Text("0", NINE_KEY_WIDTH),
                Key.Space(NINE_KEY_WIDTH),
                Key.Enter(SIDE_WIDTH),
            ),
        )
    }

    /**
     * IPA 音标：按参考键盘分成肺辅音 / 元音 / 非肺辅音 / 附加符号 / 符号五组；
     * 肺辅音再按发音方式、清浊筛选，元音按圆唇筛选，避免一屏塞满小键。
     */
    fun ipa(
        mode: Int,
        manner: Int,
        voiced: Boolean,
        rounded: Boolean,
        diacritic: Int,
    ): List<List<Key>> {
        fun symbols(values: List<String>, per: Int = 10): List<List<Key>> =
            values.chunked(per).map { row -> row.map(::ipaKey) }

        val modes = listOf(
            Key.IpaAction("mode:0", if (mode == 0) "·肺" else "肺", 1.4f),
            Key.IpaAction("mode:1", if (mode == 1) "·元" else "元", 1.4f),
            Key.IpaAction("mode:2", if (mode == 2) "·非" else "非", 1.4f),
            Key.IpaAction("mode:3", if (mode == 3) "·附" else "附", 1.4f),
            Key.IpaAction("mode:4", if (mode == 4) "·符" else "符", 1.4f),
            Key.IpaAction("voicing", if (voiced) "浊" else "清", 1.5f),
            Key.IpaAction("rounding", if (rounded) "圆" else "不圆", 1.5f),
        )
        val rows = mutableListOf<List<Key>>(modes)
        when (mode) {
            0 -> {
                val names = IPA_MANNERS
                rows += names.mapIndexed { index, name ->
                    Key.IpaAction("manner:$index", if (index == manner) "·$name" else name)
                }
                val pair = IPA_PULMONIC[manner]
                rows += symbols(if (voiced && pair.second.isNotEmpty()) pair.second else pair.first)
            }
            1 -> rows += symbols(if (rounded) IPA_VOWELS_ROUNDED else IPA_VOWELS_UNROUNDED)
            2 -> {
                rows += listOf(
                    Key.IpaAction("manner:0", if (manner == 0) "·内爆" else "内爆", 3f),
                    Key.IpaAction("manner:1", if (manner == 1) "·符号" else "符号", 3f),
                )
                rows += symbols(if (manner == 0) IPA_IMPLOSIVES else IPA_NONPULMONIC)
            }
            3 -> {
                rows += (0..6).map { index ->
                    Key.IpaAction("diacritic:$index", if (index == diacritic) "·${index + 1}" else "${index + 1}", 1.4f)
                }
                rows += IPA_DIACRITICS[diacritic].chunked(8).map { row ->
                    row.map { value ->
                        val mark = value.replace("◌", "").replace("CGJ", "\u034F")
                        Key.Text(mark, label = "a${ipaDisplayText(mark)}")
                    }
                }
            }
            else -> rows += symbols(IPA_SYMBOLS, per = 8)
        }
        rows += listOf<Key>(
            Key.Letters("ABC", 1.5f),
            Key.Backspace,
            Key.Space(4f),
            Key.Enter(),
        )
        return rows
    }

    private val IPA_MANNERS = listOf("鼻", "塞", "咝", "非咝", "近", "闪", "颤", "边擦", "边近", "边闪")

    /**
     * Doulos SIL 覆盖 IPA 与 Latin Extended-G；只有少数组合符没有字形。
     * 键值仍送原始 Unicode，键面只把这些组合符从显示里去掉。
     */
    private val IPA_DISPLAY = mapOf<String, String>(
        "̀" to "", "́" to "", "͂" to "", "̓" to "", "̈́" to "", "ͅ" to "",
    )

    private fun ipaDisplayText(value: String): String {
        val out = StringBuilder()
        var index = 0
        while (index < value.length) {
            val codePoint = value.codePointAt(index)
            val piece = String(Character.toChars(codePoint))
            out.append(IPA_DISPLAY[piece] ?: piece)
            index += Character.charCount(codePoint)
        }
        return out.toString().ifEmpty { "?" }
    }

    private fun ipaKey(value: String): Key.Text {
        val display = ipaDisplayText(value)
        return if (display == value) Key.Text(value) else Key.Text(value, label = display)
    }

    private val IPA_PULMONIC: List<Pair<List<String>, List<String>>> = listOf(
        listOf(
            "m̥", "ɱ̊", "n̪̊", "n̥", "n̠̊", "ɳ̊", "ɲ̊", "ŋ̊", "ɴ̥",
        ) to listOf("m", "ɱ", "n̼", "n̪", "n", "n̠", "ɳ", "ɲ", "ŋ", "ɴ"),
        listOf("p", "p̪", "t̼", "t̪", "t", "ʈ", "c", "k", "q", "ʡ", "ʔ") to
            listOf("b", "b̪", "d̼", "d̪", "d", "ɖ", "ɟ", "g", "ɢ"),
        listOf("s̪", "s", "ʃ", "ʂ", "ɕ") to listOf("z̪", "z", "ʒ", "ʐ", "ʑ"),
        listOf("ɸ", "f", "θ̼", "θ", "θ̠", "ɹ̠̊˔", "ɻ̊˔", "ç", "x", "χ", "ħ", "h") to
            listOf("β", "v", "ð̼", "ð", "ð̠", "ɹ̠˔", "ɻ˔", "ʝ", "ɣ", "ʁ", "ʕ", "ɦ"),
        listOf("β̞", "ʋ", "ð̞", "ɹ", "ɹ̠", "ɻ", "j", "ɰ", "ʁ̞", "ʔ̞") to emptyList(),
        listOf("ⱱ̟", "ⱱ", "ɾ̼", "ɾ", "ɽ", "ɢ̆", "ʡ̮") to emptyList(),
        listOf("ʙ̥", "r̥", "ɽ̊r̥", "ʀ̥", "ʜ") to listOf("ʙ", "r", "ɽr", "ʀ", "ʢ"),
        listOf("ɬ̪", "ɬ", "ꞎ", "𝼆", "𝼄") to listOf("ɮ", "𝼅", "ʎ̝", "ʟ̝"),
        listOf("l̥", "ɭ̊", "ʎ̥", "ʟ̥") to listOf("l̪", "l", "l̠", "ɭ", "ʎ", "ʟ", "ʟ̠"),
        listOf("ɺ", "𝼈", "ʎ̮", "ʟ̆") to emptyList(),
    )

    private val IPA_VOWELS_UNROUNDED = listOf(
        "i", "ɨ", "ɯ", "ɪ", "e", "ɘ", "ɤ", "e̞", "ə", "ɤ̞",
        "ɛ", "ɜ", "ʌ", "æ", "ɐ", "a", "ä", "ɑ",
    )
    private val IPA_VOWELS_ROUNDED = listOf(
        "y", "ʉ", "u", "ʏ", "ʊ", "ø", "ɵ", "o", "ø̞", "ə",
        "o̞", "œ", "ɞ", "ɔ", "ɐ", "ɶ", "ɒ",
    )
    private val IPA_IMPLOSIVES = listOf("ɓ", "ɗ", "ʄ", "ɠ", "ʛ", "ɓ̥", "ɗ̥", "ᶑ̊", "ʄ̊", "ɠ̊", "ʛ̥")
    private val IPA_NONPULMONIC = listOf("ʼ", "ʘ", "ǀ", "ǃ", "𝼊", "ǂ", "ǁ")
    private val IPA_DIACRITICS = listOf(
        listOf("◌̀", "◌́", "◌̂", "◌̃", "◌̄", "◌̅", "◌̆", "◌̇", "◌̈", "◌̉", "◌̊", "◌̋", "◌̌", "◌̍", "◌̎", "◌̏"),
        listOf("◌̐", "◌̑", "◌̒", "◌̓", "◌̔", "◌̕", "◌̖", "◌̗", "◌̘", "◌̙", "◌̚", "◌̛", "◌̜", "◌̝", "◌̞", "◌̟"),
        listOf("◌̠", "◌̡", "◌̢", "◌̣", "◌̤", "◌̥", "◌̦", "◌̧", "◌̨", "◌̩", "◌̪", "◌̫", "◌̬", "◌̭", "◌̮", "◌̯"),
        listOf("◌̰", "◌̱", "◌̲", "◌̳", "◌̴", "◌̵", "◌̶", "◌̷", "◌̸", "◌̹", "◌̺", "◌̻", "◌̼", "◌̽", "◌̾", "◌̿"),
        listOf("◌̀", "◌́", "◌͂", "◌̓", "◌̈́", "◌ͅ", "◌͆", "◌͇", "◌͈", "◌͉", "◌͊", "◌͋", "◌͌", "◌͍", "◌͎", "CGJ"),
        listOf("◌͐", "◌͑", "◌͒", "◌͓", "◌͔", "◌͕", "◌͖", "◌͗", "◌͘", "◌͙", "◌͚", "◌͛", "◌͜◌", "◌͝◌", "◌͞◌", "◌͟◌"),
        listOf("◌͠◌", "◌͡◌", "◌͢◌", "◌ͣ", "◌ͤ", "◌ͥ", "◌ͦ", "◌ͧ", "◌ͨ", "◌ͩ", "◌ͪ", "◌ͫ", "◌ͬ", "◌ͭ", "◌ͮ", "◌ͯ"),
    )
    private val IPA_SYMBOLS = listOf("[", "]", "/", "{", "}", "(", ")", "⸨", "⸩", "ː", "|", "꜒", "꜓", "꜔", "꜕", "꜖")


    /**
     * unt 方案（古籍字頻統計 unt 列）：键面就是该方案用到的符号，按下原样上屏。
     * 组合符 `̙` 的键面画成 `a̙`，键值只送组合符本身。
     */
    fun unt(): List<List<Key>> {
        fun row(chars: String): List<Key> = chars.split(' ').map { Key.Text(it) }
        return listOf(
            row("e ʔ w k h t ŋ n j ɑ"),
            row("ə ʰ i ˤ u m o p a s"),
            row("d l ɹ ɣ ɕ ɨ b x ʂ z"),
            listOf(
                Key.Text("ɡ"), Key.Text("ʈ"), Key.Text("ɖ"), Key.Text("ʑ"),
                Key.Text("ɲ"), Key.Text("ʐ"), Key.Text("ɳ"),
                Key.Text("\u0319", label = "a\u0319"),
                Key.Text("/"), Key.Text("·"),
            ),
            listOf<Key>(
                Key.Letters("ABC", 2f),
                Key.Backspace,
                Key.Space(4f),
                Key.Enter(),
            ),
        )
    }

    /**
     * 中文通用羅馬字（古籍字頻統計「中文通用羅馬字」列）：键面即方案符号，
     * 组合符 `̀ ́ ̥ ̧ ̱` 的键面画在示例字母上，键值只送组合符。
     */
    fun gcr(): List<List<Key>> {
        fun key(main: String, vararg alternates: String) = Key.Text(
            main,
            hint = alternates.firstOrNull().orEmpty(),
            alternates = alternates.toList(),
        )
        return listOf(
            listOf(
                key("i", "ɨ", "ị", "ɏ"),
                key("u"),
                key("k"),
                key("o", "œ", "ȝ"),
                key("e", "ę"),
                key("ŋ"),
                key("h"),
                key("n", "ņ"),
                key("j", "ʒ"),
                key("a", "ḁ"),
            ),
            listOf(
                key("ꝩ"),
                key("t", "ţ"),
                key("m", "ḑ"),
                key("y", "ỵ"),
                key("p"),
                key("l"),
                key("q"),
                key("c", "ç"),
                key("w"),
                key("f"),
            ),
            listOf(
                key("g"),
                key("d"),
                key("v"),
                key("s"),
                key("x"),
                key("ə"),
                key("ş"),
                key("æ"),
                key("r"),
                key("b"),
            ),
            listOf(
                Key.Text("q", label = "á"),
                Key.Text("\u0300", label = "\u25CC\u0300"),
                Key.Text("\u0301", label = "\u25CC\u0301"),
                Key.Text("\u0325", label = "\u25CC\u0325"),
                Key.Text("\u0327", label = "\u25CC\u0327"),
                Key.Text("\u0331", label = "\u25CC\u0331"),
                Key.Text("h", label = "à"),
            ),
            listOf<Key>(
                Key.Letters("ABC", 1.7f),
                Key.Backspace,
                Key.Space(4f),
                Key.Enter(),
            ),
        )
    }

    /** 常用符号页。英文：数字、常用符号，「更多」进完整符号面板，「ABC」回字母。 */
    fun symbols(chinese: Boolean, fullWidth: Boolean = true): List<List<Key>> = if (chinese) {
        listOf(
            "1234567890".map { Key.Text(it.toString()) },
            "、；：？！（）《》～".map { Key.Text(it.toString()) },
            listOf<Key>(Key.Text("…", 1.5f)) + "“”‘’【】—".map { Key.Text(it.toString()) } + Key.Backspace,
            bottomRow(Key.Symbols("符", 1.25f), Key.Letters("返回"), chinese = true, fullWidth),
        )
    } else {
        listOf(
            "1234567890".map { Key.Text(it.toString()) },
            "@#\$_&-+()/".map { Key.Text(it.toString()) },
            listOf<Key>(Key.Symbols("更多", 1.5f)) + "*\"':;!?".map { Key.Text(it.toString()) } + Key.Backspace,
            bottomRow(Key.Letters("ABC", 1.25f), Key.NumberPage("123"), chinese = false),
        )
    }
}
