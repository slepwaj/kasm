package app.kasm.inputmethod.settings.page

import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.os.SystemClock
import android.text.Editable
import android.text.InputType
import android.text.TextWatcher
import android.view.Gravity
import android.view.View
import android.view.ViewGroup.LayoutParams.MATCH_PARENT
import android.view.ViewGroup.LayoutParams.WRAP_CONTENT
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputMethodManager
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView
import app.kasm.inputmethod.R
import app.kasm.inputmethod.panel.base.PanelIcons.Icon
import app.kasm.inputmethod.settings.Settings
import app.kasm.inputmethod.settings.home.ImeState
import app.kasm.inputmethod.settings.widget.PracticeText
import app.kasm.inputmethod.settings.widget.SettingsWidgets
import app.kasm.inputmethod.theme.AppColors
import java.io.IOException
import java.util.Random

/**
 * 打字练习：随机抽几句拼成一段，用户跟着打，按正确字数和用时算速度、准确率。
 * 输入框用的是系统输入法（KASM 选中时就是 KASM 键盘）。
 */
class PracticeActivity : PageActivity() {
    override val title = R.string.practice_title

    private data class PracticeLine(
        val corpus: String,
        val text: String,
        val traditional: String,
        val readings: Map<String, List<String>>,
    )

    private data class Target(val text: String, val readings: List<String>)

    private val colors get() = ui.colors

    private lateinit var targetText: String
    private lateinit var targetReadings: List<String>
    private lateinit var targetView: PracticeText
    private lateinit var input: EditText
    private lateinit var timeValue: TextView
    private lateinit var speedValue: TextView
    private lateinit var accuracyValue: TextView
    private lateinit var progressValue: TextView
    private lateinit var resultView: TextView
    private lateinit var simplifiedChip: TextView
    private lateinit var traditionalChip: TextView

    private var startedAt = 0L
    private var finishedAt = 0L
    private var finished = false
    private var lastTarget = ""
    private var scriptTraditional = false

    /** 注音跟随键盘当前的拼音方案；练习通字时上面显示通字，练全拼时显示拼音。 */
    private val annotationScheme by lazy {
        when (Settings.load(this).romanization) {
            "tungdzih", "tupa", "shanggu", "unt", "gcr", "jyutping" ->
                Settings.load(this).romanization
            else -> "pinyin"
        }
    }

    private val random = Random()
    private val handler = Handler(Looper.getMainLooper())
    private val ticker = object : Runnable {
        override fun run() {
            if (!finished && startedAt != 0L) {
                renderStats()
                handler.postDelayed(this, 500)
            }
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        val savedTarget = savedInstanceState?.getString(KEY_TARGET)
        val savedReadings = savedInstanceState?.getString(KEY_READINGS)
        scriptTraditional = savedInstanceState?.getBoolean(KEY_TRADITIONAL)
            ?: Settings.load(this).traditional
        targetText = savedTarget ?: FALLBACK_TARGET
        targetReadings = savedReadings?.split(READING_SEPARATOR)
            ?: List(targetText.length) { "" }
        super.onCreate(savedInstanceState)
        if (savedTarget == null) {
            val target = newTarget()
            targetText = target.text
            targetReadings = target.readings
        }
        savedInstanceState?.let { saved ->
            startedAt = saved.getLong(KEY_STARTED, 0L)
            finishedAt = saved.getLong(KEY_FINISHED_AT, 0L)
            finished = saved.getBoolean(KEY_FINISHED, false)
            input.setText(saved.getString(KEY_TYPED).orEmpty())
            input.isEnabled = !finished
            if (finished) {
                resultView.visibility = View.VISIBLE
            }
        }
        render()
    }

    override fun onResume() {
        super.onResume()
        if (!::input.isInitialized) return
        if (!finished) {
            input.requestFocus()
            input.postDelayed({
                getSystemService(InputMethodManager::class.java)
                    .showSoftInput(input, InputMethodManager.SHOW_IMPLICIT)
            }, 120)
            scheduleTicker()
        }
    }

    override fun onPause() {
        handler.removeCallbacks(ticker)
        super.onPause()
    }

    override fun onSaveInstanceState(outState: Bundle) {
        outState.putString(KEY_TARGET, targetText)
        outState.putString(KEY_READINGS, targetReadings.joinToString(READING_SEPARATOR))
        if (::input.isInitialized) outState.putString(KEY_TYPED, committedText())
        outState.putLong(KEY_STARTED, startedAt)
        outState.putLong(KEY_FINISHED_AT, finishedAt)
        outState.putBoolean(KEY_FINISHED, finished)
        outState.putBoolean(KEY_TRADITIONAL, scriptTraditional)
        super.onSaveInstanceState(outState)
    }

    override fun content(): List<View> {
        val views = mutableListOf<View>()
        if (!ImeState.of(this).ready) views += spaced(setupCard())
        views += spaced(targetCard())
        views += spaced(statsCard())
        views += spaced(inputCard())
        views += spaced(buttonsCard())
        views += spaced(resultCard())
        return views
    }

    private fun setupCard() = ui.card(
        ui.entry(
            Icon.KEYBOARD,
            AppColors.JADE,
            R.string.practice_enable,
            getString(R.string.practice_enable_note),
        ) {
            if (ImeState.of(this).enabled) {
                ImeState.showPicker(this)
            } else {
                ImeState.openSystemSettings(this)
            }
        },
    )

    override fun topBarActions(): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        simplifiedChip = scriptChip(R.string.practice_script_simplified, false)
        traditionalChip = scriptChip(R.string.practice_script_traditional, true)
        addView(simplifiedChip)
        addView(traditionalChip, LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
            marginStart = ui.dp(6)
        })
        styleScriptChips()
    }

    private fun targetCard(): View {
        targetView = PracticeText(this, colors)
        return ui.card(LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(ui.dp(20), ui.dp(16), ui.dp(20), ui.dp(18))
            addView(ui.text(getString(R.string.practice_target_label), 12.5f, colors.secondary))
            addView(ui.text(getString(R.string.practice_scheme, schemeLabel()), 11.5f, colors.faint).apply {
                setPadding(0, ui.dp(4), 0, 0)
            })
            addView(targetView, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = ui.dp(8)
            })
        })
    }

    private fun scriptChip(label: Int, traditional: Boolean) =
        ui.text(getString(label), 12f, colors.secondary, bold = true).apply {
            gravity = Gravity.CENTER
            setPadding(ui.dp(12), ui.dp(6), ui.dp(12), ui.dp(6))
            setOnClickListener { setScript(traditional) }
        }

    private fun styleScriptChips() {
        if (!::simplifiedChip.isInitialized) return
        for ((chip, traditional) in listOf(simplifiedChip to false, traditionalChip to true)) {
            val selected = scriptTraditional == traditional
            chip.background = ui.rounded(if (selected) colors.brand else colors.brandSoft, 14f)
            chip.setTextColor(if (selected) colors.onBrand else colors.brand)
        }
    }

    private fun setScript(traditional: Boolean) {
        if (scriptTraditional == traditional) return
        scriptTraditional = traditional
        Settings.preferences(this).edit()
            .putBoolean(Settings.KEY_TRADITIONAL, traditional)
            .apply()
        styleScriptChips()
        newPassage()
    }

    private fun statsCard(): View {
        val row = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(ui.dp(6), ui.dp(16), ui.dp(6), ui.dp(14))
        }
        timeValue = stat(row, R.string.practice_time)
        speedValue = stat(row, R.string.practice_speed)
        accuracyValue = stat(row, R.string.practice_accuracy)
        progressValue = stat(row, R.string.practice_progress)
        return ui.card(row)
    }

    private fun stat(row: LinearLayout, label: Int): TextView {
        val value = ui.text("—", 18f, colors.text, bold = true).apply {
            gravity = Gravity.CENTER
        }
        row.addView(LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            addView(value)
            addView(ui.text(getString(label), 11.5f, colors.secondary).apply {
                gravity = Gravity.CENTER
                setPadding(0, ui.dp(6), 0, 0)
            })
        }, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f))
        return value
    }

    private fun inputCard(): View {
        input = EditText(this).apply {
            setHint(R.string.practice_input_hint)
            setHintTextColor(colors.faint)
            setTextColor(colors.text)
            textSize = 18f
            background = null
            gravity = Gravity.TOP or Gravity.START
            inputType = InputType.TYPE_CLASS_TEXT or
                InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            imeOptions = EditorInfo.IME_FLAG_NO_EXTRACT_UI or EditorInfo.IME_FLAG_NO_FULLSCREEN
            isSingleLine = false
            minLines = 3
            maxLines = 8
            importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS
            setPadding(ui.dp(20), ui.dp(16), ui.dp(20), ui.dp(16))
            addTextChangedListener(object : TextWatcher {
                override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
                override fun afterTextChanged(s: Editable?) = render()
            })
            setOnClickListener { showKeyboard() }
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT)
        }
        return ui.card(input)
    }

    private fun buttonsCard() = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        addView(button(R.string.practice_new, colors.brand, colors.onBrand) { newPassage() },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginEnd = ui.dp(6) })
        addView(button(R.string.practice_restart, colors.brandSoft, colors.brand) { restart() },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginStart = ui.dp(6) })
    }

    private fun button(label: Int, background: Int, foreground: Int, onClick: () -> Unit) =
        ui.text(getString(label), 15f, foreground, bold = true).apply {
            gravity = Gravity.CENTER
            this.background = ui.rounded(background, 18f)
            setPadding(ui.dp(12), ui.dp(13), ui.dp(12), ui.dp(13))
            setOnClickListener { onClick() }
        }

    private fun resultCard(): View {
        resultView = ui.text("", 13f, colors.brand, bold = true).apply {
            visibility = View.GONE
            setPadding(ui.dp(4), ui.dp(2), ui.dp(4), 0)
        }
        return resultView
    }

    private fun showKeyboard() {
        if (finished) return
        input.requestFocus()
        getSystemService(InputMethodManager::class.java)
            .showSoftInput(input, InputMethodManager.SHOW_IMPLICIT)
    }

    private fun newPassage() {
        val target = newTarget()
        targetText = target.text
        targetReadings = target.readings
        startedAt = 0L
        finishedAt = 0L
        finished = false
        input.isEnabled = true
        input.setText("")
        resultView.visibility = View.GONE
        render()
    }

    private fun restart() {
        startedAt = 0L
        finishedAt = 0L
        finished = false
        input.isEnabled = true
        input.setText("")
        resultView.visibility = View.GONE
        render()
    }

    private fun render() {
        if (!::input.isInitialized) return
        val typed = committedText()
        if (!finished && typed.isNotEmpty() && startedAt == 0L) {
            startedAt = SystemClock.elapsedRealtime()
            scheduleTicker()
        }
        if (!finished && typed.length >= targetText.length && typed.startsWith(targetText)) {
            finished = true
            finishedAt = SystemClock.elapsedRealtime()
            input.isEnabled = false
            handler.removeCallbacks(ticker)
            resultView.visibility = View.VISIBLE
        }
        targetView.update(targetText, targetReadings, typed)
        renderStats()
    }

    private fun renderStats() {
        if (!::timeValue.isInitialized) return
        val typed = committedText()
        val compared = minOf(typed.length, targetText.length)
        var correct = 0
        for (index in 0 until compared) {
            if (typed[index] == targetText[index]) correct++
        }
        val elapsed = elapsedSeconds()
        val speed = if (elapsed > 0.0) correct * 60.0 / elapsed else 0.0
        val accuracy = if (typed.isEmpty()) 100.0 else correct * 100.0 / typed.length
        timeValue.text = getString(R.string.practice_seconds, elapsed)
        speedValue.text = getString(R.string.practice_speed_value, speed)
        accuracyValue.text = getString(R.string.practice_percent, accuracy)
        progressValue.text = getString(R.string.practice_progress_value, compared, targetText.length)
        if (finished) {
            resultView.text = getString(
                R.string.practice_done,
                elapsedSeconds(),
                if (elapsed > 0.0) correct * 60.0 / elapsed else 0.0,
                accuracy,
            )
        }
    }

    private fun elapsedSeconds(): Double {
        if (startedAt == 0L) return 0.0
        val end = if (finishedAt != 0L) finishedAt else SystemClock.elapsedRealtime()
        return (end - startedAt) / 1000.0
    }

    /** 输入法组句时输入框里会有带 composing span 的拼音，只比较已经上屏的部分。 */
    private fun committedText(): String {
        val editable = input.text
        val start = BaseInputConnection.getComposingSpanStart(editable)
        val end = BaseInputConnection.getComposingSpanEnd(editable)
        return if (start >= 0 && end > start && end <= editable.length) {
            editable.toString().removeRange(start, end)
        } else {
            editable.toString()
        }
    }

    private fun scheduleTicker() {
        handler.removeCallbacks(ticker)
        handler.post(ticker)
    }

    private fun newTarget(): Target {
        var result = fallbackTarget()
        for (attempt in 0 until 4) {
            result = buildTarget()
            if (result.text != lastTarget) break
        }
        lastTarget = result.text
        return result
    }

    private fun buildTarget(): Target {
        val all = lineCache
        val preferred = all.filter { it.corpus == preferredCorpus() }
        val base = preferred.ifEmpty { all }
        val covered = base.filter(::covered)
        val pool = covered.ifEmpty { base }
        if (pool.isEmpty()) return fallbackTarget()
        val goal = 30 + random.nextInt(31)
        val picked = StringBuilder()
        val readings = mutableListOf<String>()
        for (line in pool.shuffled(random)) {
            val text = if (scriptTraditional) line.traditional else line.text
            picked.append(text)
            val lineReadings = line.readings[annotationScheme].orEmpty()
            for (index in text.indices) {
                readings += lineReadings.getOrElse(index) { "" }
            }
            if (picked.length >= goal) break
        }
        return Target(picked.toString(), readings)
    }

    /** 粤拼用粤语语料；上古音 / 中古音等用文言文；全拼 / 通字用现代文。 */
    private fun preferredCorpus(): String = when (annotationScheme) {
        "jyutping" -> "cantonese"
        "tupa", "shanggu", "unt", "gcr" -> "classical"
        else -> "modern"
    }

    /** 当前方案对这句每个汉字都有读音，练习时才不会出现空白注音和打不出的字。 */
    private fun covered(line: PracticeLine): Boolean {
        val readings = line.readings[annotationScheme] ?: return false
        return line.text.indices.all { index ->
            val char = line.text[index]
            char !in '\u4e00'..'\u9fff' || readings.getOrNull(index)?.isNotEmpty() == true
        }
    }

    private fun fallbackTarget(): Target {
        val text = if (scriptTraditional) FALLBACK_TRADITIONAL else FALLBACK_TARGET
        return Target(text, List(text.length) { "" })
    }

    private fun schemeLabel(): String = getString(
        when (annotationScheme) {
            "tungdzih" -> R.string.practice_scheme_tungdzih
            "tupa" -> R.string.practice_scheme_tupa
            "shanggu" -> R.string.practice_scheme_shanggu
            "unt" -> R.string.practice_scheme_unt
            "gcr" -> R.string.practice_scheme_gcr
            "jyutping" -> R.string.practice_scheme_jyutping
            else -> R.string.practice_scheme_pinyin
        },
    )

    private val lineCache by lazy { loadLines() }

    private fun loadLines(): List<PracticeLine> {
        val table = try {
            assets.open("data/practice/readings.tsv").bufferedReader().useLines { lines ->
                lines.filter { it.isNotBlank() && !it.startsWith("#") }
                    .mapNotNull { line ->
                        val fields = line.split('\t')
                        val corpus = fields.getOrNull(0)?.takeIf { it.isNotEmpty() } ?: return@mapNotNull null
                        val text = fields.getOrNull(1)?.takeIf { it.isNotEmpty() } ?: return@mapNotNull null
                        val traditional = fields.getOrNull(2)?.takeIf { it.isNotEmpty() } ?: text
                        val readings = SCHEMES.mapIndexed { index, scheme ->
                            val values = fields.getOrNull(index + 3)?.split('|').orEmpty()
                            scheme to List(text.length) { values.getOrElse(it) { "" } }
                        }.toMap()
                        PracticeLine(corpus, text, traditional, readings)
                    }
                    .toList()
            }
        } catch (_: IOException) {
            emptyList()
        }
        if (table.isNotEmpty()) return table
        return try {
            assets.open("data/practice/sentences.txt").bufferedReader().useLines { lines ->
                lines.map { it.trim() }
                    .filter { it.isNotEmpty() && !it.startsWith("#") }
                    .map { PracticeLine("modern", it, it, emptyMap()) }
                    .toList()
            }
        } catch (_: IOException) {
            emptyList()
        }
    }

    companion object {
        private const val KEY_TARGET = "practice_target"
        private const val KEY_READINGS = "practice_readings"
        private const val KEY_TRADITIONAL = "practice_traditional"
        private const val KEY_TYPED = "practice_typed"
        private const val KEY_STARTED = "practice_started"
        private const val KEY_FINISHED_AT = "practice_finished_at"
        private const val KEY_FINISHED = "practice_finished"
        private const val READING_SEPARATOR = "\u001f"
        private const val FALLBACK_TARGET = "今天的天气很好，适合出去走走。"
        private const val FALLBACK_TRADITIONAL = "今天的天氣很好，適合出去走走。"
        private val SCHEMES = listOf(
            "pinyin",
            "tungdzih",
            "tupa",
            "shanggu",
            "unt",
            "gcr",
            "jyutping",
        )
    }
}
