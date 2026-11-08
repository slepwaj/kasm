//! KASM addon 工厂。
#include "kasm.h"
namespace fcitx {
class KasmFactory final : public AddonFactory {
public:
    AddonInstance *create(AddonManager *manager) override { return new KasmEngine(manager); }
};
}
FCITX_ADDON_FACTORY_V2(kasm, fcitx::KasmFactory)
