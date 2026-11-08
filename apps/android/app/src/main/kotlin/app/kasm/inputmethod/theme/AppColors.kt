package app.kasm.inputmethod.theme

import android.content.Context

/**
 * App 页面的配色，与键盘的 [Palette] 分开：忧郁的深灰黑底、金属银文字与边框、
 * 顶部一层冷灰渐变；功能入口图标块也用低饱和的钢灰，不出现荧光色。键盘主题不受影响。
 */
class AppColors(
    val background: Int,
    /** 页面顶部渐变的起始色，往下渐变到 [background]。 */
    val wash: Int,
    val surface: Int,
    val text: Int,
    val secondary: Int,
    val faint: Int,
    val divider: Int,
    val brand: Int,
    val onBrand: Int,
    /** 品牌色的浅底：选中的单选外圈、标签底。 */
    val brandSoft: Int,
    /** 功能入口图标块的颜色，按顺序取。 */
    val tiles: List<Int>,
    val dark: Boolean,
) {
    companion object {
        fun of(@Suppress("UNUSED_PARAMETER") context: Context): AppColors = GRAPHITE

        const val JADE = 0
        const val BLUE = 1
        const val ORANGE = 2
        const val PURPLE = 3
        const val PINK = 4
        const val CYAN = 5

        private val TILES = listOf(
            0xFF6E7B88.toInt(),
            0xFF8C99A6.toInt(),
            0xFFA7B2BD.toInt(),
            0xFF5F6B78.toInt(),
            0xFF7E8A96.toInt(),
            0xFF99A6B3.toInt(),
        )

        /** 金属石墨：黑底、银字、钢灰描边；所有彩色图标块也压成冷灰。 */
        private val GRAPHITE = AppColors(
            background = 0xFF0A0B0D.toInt(),
            wash = 0xFF262C33.toInt(),
            surface = 0xFF14171B.toInt(),
            text = 0xFFDCE1E6.toInt(),
            secondary = 0xFF9AA3AD.toInt(),
            faint = 0xFF5B646E.toInt(),
            divider = 0xFF23282E.toInt(),
            brand = 0xFFC6CED6.toInt(),
            onBrand = 0xFF0A0B0D.toInt(),
            brandSoft = 0xFF242A31.toInt(),
            tiles = TILES,
            dark = true,
        )
    }
}
