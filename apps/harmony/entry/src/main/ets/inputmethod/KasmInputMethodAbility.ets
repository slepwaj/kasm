import InputMethodExtensionAbility from '@ohos.InputMethodExtensionAbility';
import inputMethodEngine from '@ohos.inputMethodEngine';
import type Want from '@ohos.app.ability.Want';
import { hilog } from '@kit.PerformanceAnalysisKit';

const DOMAIN = 0x0000;
const TAG = 'KasmIME';

/**
 * KASM 输入法扩展：系统在输入框获得焦点时拉起，创建软键盘 panel 并加载
 * `pages/Keyboard`。引擎下一步接到 Rust（NAPI）上，这里先跑通上屏链路。
 */
export default class KasmInputMethodAbility extends InputMethodExtensionAbility {
  private panel?: inputMethodEngine.Panel;

  onCreate(want: Want): void {
    hilog.info(DOMAIN, TAG, 'onCreate');
    const ability = inputMethodEngine.getInputMethodAbility();
    ability.on('inputStart', async (controller, client) => {
      AppStorage.setOrCreate('kasm.client', client);
      try {
        let panel = this.panel;
        if (panel === undefined) {
          panel = await ability.createPanel(this.context, {
            type: inputMethodEngine.PanelType.SOFT_KEYBOARD,
            flag: inputMethodEngine.PanelFlag.FLG_FIXED
          });
          await panel.setUiContent('pages/Keyboard');
          this.panel = panel;
        }
        await panel.show();
      } catch (error) {
        hilog.error(DOMAIN, TAG, 'create panel failed: %{public}s', JSON.stringify(error));
      }
    });
    ability.on('inputStop', () => {
      AppStorage.setOrCreate('kasm.client', undefined);
    });
  }

  onDestroy(): void {
    hilog.info(DOMAIN, TAG, 'onDestroy');
    this.panel?.hide();
    this.panel = undefined;
  }
}
