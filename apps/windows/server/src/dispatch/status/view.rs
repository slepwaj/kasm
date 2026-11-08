use kasm_platform::Appearance;
use kasm_render::Mode;

/// 状态条一次要显示的内容。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusView {
    /// 英文模式（`false` 中文）。
    pub english: bool,

    /// 开着双拼时的方案名，中文格里跟在「中」后面。
    pub scheme: Option<String>,

    /// 方案格显示的字：拼 / 粤 / 通 / un / tp。
    pub scheme_cell: String,

    /// 方案格是不是非全拼（决定高亮色）。
    pub scheme_active: bool,

    /// 当前输出繁体（状态条的「简 / 繁」格显示繁，点一下切回简体）。
    pub traditional: bool,

    /// 当前模式的全角标点开着（中英各记一份配置）；关着时格子显示 `,.` 画成灰的。
    pub full_width: bool,

    /// 外观：跟随系统 / 浅色 / 深色。
    pub appearance: Appearance,

    /// 配置里记住的内容左上角物理像素；`None` 首次按屏幕右下角摆。
    pub anchor: Option<(i32, i32)>,

    /// 大小（百分比，100 为原大），与候选字号无关。
    pub size: u8,

    /// 输入状态，给节点树画法的主题显示（与候选窗口同一份）。
    pub mode: Mode,
}
