package app.kasm.inputmethod.settings.widget

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import android.util.TypedValue
import android.view.View
import app.kasm.inputmethod.theme.AppColors

/**
 * 带注音的目标文字：每个字上面是当前方案的读音，下面是汉字；按已打内容染对 / 错 / 当前位置。
 * 逐字排版，读音长的字格自动变宽，整段在练习页里按宽度换行。
 */
class PracticeText(context: Context, private val colors: AppColors) : View(context) {
    private data class Cell(val index: Int, val x: Float, val y: Float, val width: Float)

    private val density = resources.displayMetrics.density
    private val spScale = TypedValue.applyDimension(
        TypedValue.COMPLEX_UNIT_SP,
        1f,
        resources.displayMetrics,
    )
    private val readingPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textSize = 10f * spScale
        typeface = android.graphics.Typeface.DEFAULT_BOLD
    }
    private val charPaint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        textSize = 19f * spScale
    }
    private val backgroundPaint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val padding = (6f * density).toInt()

    private var chars: List<Char> = emptyList()
    private var readings: List<String> = emptyList()
    private var typed = ""
    private var cells: List<Cell> = emptyList()
    private var contentHeight = 0
    private var laidOutWidth = 0

    private val readingHeight get() = readingPaint.descent() - readingPaint.ascent()
    private val charHeight get() = charPaint.descent() - charPaint.ascent()
    private val rowHeight get() = readingHeight + charHeight + 4f * density

    fun update(text: String, readings: List<String>, typed: String) {
        val nextChars = text.toList()
        val relayout = nextChars != chars || readings != this.readings
        chars = nextChars
        this.readings = readings
        this.typed = typed
        contentDescription = text
        if (relayout) {
            cells = emptyList()
            requestLayout()
        }
        invalidate()
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val width = MeasureSpec.getSize(widthMeasureSpec)
        if (width > 0) layoutCells(width)
        setMeasuredDimension(width, maxOf(contentHeight, (32f * density).toInt()))
    }

    override fun onSizeChanged(width: Int, height: Int, oldWidth: Int, oldHeight: Int) {
        super.onSizeChanged(width, height, oldWidth, oldHeight)
        if (width > 0 && width != laidOutWidth) {
            layoutCells(width)
            invalidate()
        }
    }

    private fun layoutCells(width: Int) {
        laidOutWidth = width
        if (chars.isEmpty()) {
            cells = emptyList()
            contentHeight = 0
            return
        }
        val left = padding.toFloat()
        val right = (width - padding).toFloat()
        var x = left
        var y = padding.toFloat()
        var row = 0f
        val laidOut = ArrayList<Cell>(chars.size)
        for (index in chars.indices) {
            val reading = readings.getOrElse(index) { "" }
            val cellWidth = maxOf(
                charPaint.measureText(chars[index].toString()),
                readingPaint.measureText(reading),
            ) + 8f * density
            if (x > left && x + cellWidth > right) {
                x = left
                y += row
                row = 0f
            }
            laidOut += Cell(index, x, y, cellWidth)
            row = maxOf(row, rowHeight)
            x += cellWidth
        }
        cells = laidOut
        contentHeight = (y + row + padding).toInt()
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        if (cells.isEmpty()) return
        val readingOffset = readingHeight
        val charBox = charHeight + 4f * density
        for (cell in cells) {
            val char = chars[cell.index]
            val reading = readings.getOrElse(cell.index) { "" }
            val typedHere = cell.index < typed.length
            val current = cell.index == typed.length

            if (current) {
                backgroundPaint.color = colors.brandSoft
                val top = cell.y + readingOffset
                canvas.drawRoundRect(
                    RectF(cell.x, top, cell.x + cell.width, top + charBox),
                    6f * density,
                    6f * density,
                    backgroundPaint,
                )
            }

            readingPaint.color = if (current) brightReadingColor() else readingColor()
            if (reading.isNotEmpty()) {
                val width = readingPaint.measureText(reading)
                canvas.drawText(
                    reading,
                    cell.x + (cell.width - width) / 2f,
                    cell.y - readingPaint.ascent(),
                    readingPaint,
                )
            }

            charPaint.color = when {
                typedHere && typed[cell.index] == char -> correctColor()
                typedHere -> wrongColor()
                current -> colors.brand
                else -> colors.text
            }
            val width = charPaint.measureText(char.toString())
            canvas.drawText(
                char.toString(),
                cell.x + (cell.width - width) / 2f,
                cell.y + readingOffset - charPaint.ascent(),
                charPaint,
            )
        }
    }

    private fun correctColor() = if (colors.dark) 0xFF63D69A.toInt() else 0xFF2E9E5B.toInt()

    private fun wrongColor() = if (colors.dark) 0xFFFF8A8A.toInt() else 0xFFD93F3F.toInt()

    private fun readingColor() = if (colors.dark) 0xFFFFD54F.toInt() else 0xFFAD6800.toInt()

    private fun brightReadingColor() = if (colors.dark) 0xFFFFF176.toInt() else 0xFF8A4B00.toInt()
}
