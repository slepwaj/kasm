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
     * Xina 拼音：全键盘上方多一排声调键。键面写「1 阴平」等，按下去传 `!@#$%`；
     * 数字键盘里的 1-5 也仍然可用。
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
     * IPA 音标：键面就是音标符号，按下去原样上屏（不经拼音引擎）。
     * 五排常用辅音/元音，底排留 ABC、空格、退格、回车。
     */
    fun ipa(page: Int = 0): List<List<Key>> {
        fun symbols(row: String): List<Key> = row.split(' ').map { Key.Text(it) }
        fun bottom(next: String) = listOf<Key>(
            Key.IpaPage(next, 1.5f),
            Key.Letters("ABC", 1.5f),
            Key.Backspace,
            Key.Space(3.5f),
            Key.Enter(),
        )
        return when (page) {
            1 -> listOf(
                symbols("ɸ β ʋ ɱ ɳ ɲ ɴ ʎ ɾ ʀ"),
                symbols("ʈ ɖ ɽ ɭ ɬ ɮ ɺ ɫ ɰ ɥ"),
                symbols("c ɟ ç ʝ ɣ χ ʁ ħ ʕ ɦ"),
                symbols("ʘ ǀ ǃ ǂ ǁ ɓ ɗ ʄ ɠ ʛ"),
                symbols("ɘ ɵ ɞ ɐ ɚ ɝ ɤ ɯ ɨ ʉ"),
                symbols("˥ ˦ ˧ ˨ ˩ ↗ ↘ → ‖ ‿"),
                bottom("附加"),
            )
            2 -> {
                // 组合附加符号：键面画在示例字母上（虚线圆在手机字体里常缺字），键值只送组合符本身
                fun d(mark: String): Key = Key.Text(mark, label = "a$mark")
                listOf(
                    listOf(d("\u0325"), d("\u030A"), d("\u0324"), d("\u0330"), d("\u032A"), d("\u033A"), d("\u033B"), d("\u033C"), d("\u0334"), d("\u031D")),
                    listOf(d("\u031E"), d("\u0318"), d("\u0319"), d("\u0303"), d("\u0329"), d("\u032F"), Key.Text("ʷ"), Key.Text("ʲ"), Key.Text("ˠ"), Key.Text("ˤ")),
                    listOf(d("\u031A"), Key.Text("ʰ"), Key.Text("ˀ"), Key.Text("ˈ"), Key.Text("ˌ"), Key.Text("ː"), Key.Text("ˑ"), Key.Text("."), Key.Text("‿"), Key.Text("‖")),
                    bottom("基础"),
                )
            }
            else -> listOf(
                symbols("p b t d k g ʔ m n ŋ"),
                symbols("f v θ ð s z ʃ ʒ h x"),
                symbols("tʃ dʒ r l j w ɹ ɻ ʂ ʐ"),
                symbols("i y ɨ ʉ ɯ u ɪ ʏ ʊ e"),
                symbols("ø ɛ œ ɜ ɐ a ɶ ɑ ɒ ɔ"),
                symbols("o ə ɤ ɚ ɝ æ ʌ ɑ̃ ɛ̃ ɔ̃"),
                bottom("更多"),
            )
        }
    }

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
        fun row(chars: String): List<Key> = chars.split(' ').map { Key.Text(it) }
        return listOf(
            row("i u k o e ŋ h n j a"),
            row("ꝩ t m y p l q c ç ȝ"),
            row("ị d ę s x ə ş æ ɨ b"),
            row("g ţ w ḑ f r ʒ ḁ ỵ v"),
            listOf(
                Key.Text("ɏ"), Key.Text("z"), Key.Text("œ"), Key.Text("ņ"),
                Key.Text("\u0325", label = "a\u0325"),
                Key.Text("\u0327", label = "a\u0327"),
                Key.Text("\u0331", label = "a\u0331"),
                Key.Text("/"), Key.Text("·"),
            ),
            listOf(
                Key.Text("q", label = "á"),
                Key.Text("h", label = "à"),
            ),
            listOf<Key>(
                Key.Letters("ABC", 2f),
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
