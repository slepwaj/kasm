//! 壳手里「现在装着哪套方案」：按配置的两条轴装配引擎，形码方案与码表文件都没变时不重读码表。
//!
//! 每次配置热加载都会走到这里，而大多数时候改的是别的设置；8.9 万条的码表每次重读一遍是白花的。
//! 用户双拼方案只是一张小键位表，每次都重新读，变了才换进引擎。

use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;

use kasm_core::Engine;

use crate::code_schemes::{self, CodeSchemeEntry};
use crate::shuangpin_schemes::{self, ShuangpinSchemeEntry};
use crate::{Config, DictionariesConfig, Scheme, scheme_label, scheme_name};

/// 码表与各本形码词库的修改时间与长度：都没变就当没改。
type Stamp = Vec<(Option<SystemTime>, u64)>;

/// 三套主词库的路径。
#[derive(Debug, Clone)]
struct DictionaryPaths {
    pinyin: std::path::PathBuf,
    jyutping: std::path::PathBuf,
    tungdzih: std::path::PathBuf,
    tupa: std::path::PathBuf,
    shanggu: std::path::PathBuf,
}

/// 当前装上的用户双拼方案与形码方案（含码表文件的指纹）；没选或没装上为 `None`。
#[derive(Debug, Default)]
pub struct SchemeSlot {
    shuangpin: Option<ShuangpinSchemeEntry>,

    code: Option<(CodeSchemeEntry, Stamp)>,

    /// 三套主词库（全拼 / 粤拼 / 通字）的路径；装配方在启动时告诉槽位，切换方案时在这儿换库。
    dictionary_paths: Option<DictionaryPaths>,
}

impl SchemeSlot {
    /// 告诉槽位三套主词库的位置。不设时切换方案只改引擎标志、不换库。
    pub fn set_dictionary_paths(
        &mut self,
        pinyin: std::path::PathBuf,
        jyutping: std::path::PathBuf,
        tungdzih: std::path::PathBuf,
        tupa: std::path::PathBuf,
        shanggu: std::path::PathBuf,
    ) {
        self.dictionary_paths = Some(DictionaryPaths {
            pinyin,
            jyutping,
            tungdzih,
            tupa,
            shanggu,
        });
    }

    /// 按配置装配引擎：拼音侧（`[general] scheme`：全拼 / 双拼 / 用户双拼 / 关）与
    /// 形码侧（`[general] code`，加上 `[code.<id>]` 里用户改的规则）。两边都开就是混输，见 `Engine::query_mixed`。
    ///
    /// 选了的方案找不到或读不了时警告并退回拼音（配置说五笔、引擎一个字都打不出更糟）；用户双拼找不到退回全拼。
    /// 每次都重新找方案（用户可能刚换了方案、刚放进新方案），但找到的形码方案与装着的是同一套、码表文件也没动时不重读，
    /// 只改规则。`bundled` 是随包的 `schemes` 目录。
    pub fn apply(
        &mut self,
        engine: &mut Engine,
        config: &Config,
        user_dir: Option<&Path>,
        bundled: Option<&Path>,
    ) {
        let pinyin = config.general.scheme();
        let shuangpin = config.general.custom_shuangpin();
        self.apply_shuangpin(engine, pinyin, shuangpin.as_deref(), user_dir, bundled);
        self.apply_code(engine, config, user_dir, bundled);
        // 粤拼与拼音共用同一个 Engine，主词库不同：方案在两者之间切换时把词库整个换掉，
        // 不必重启 Server（配置热加载走到这里）。启动时装配方已经装对了词库，这里只在标志不一致时动。
        // 0 = 全拼 / 双拼，1 = 粤拼，2 = 通字；三者共用同一个 Engine，主词库不同
        let wants = match pinyin {
            Scheme::Jyutping => 1,
            Scheme::Tungdzih => 2,
            Scheme::Tupa => 3,
            Scheme::Shanggu => 4,
            _ => 0,
        };
        let current = if engine.is_shanggu() {
            4
        } else if engine.is_tupa() {
            3
        } else if engine.is_tungdzih() {
            2
        } else if engine.is_jyutping() {
            1
        } else {
            0
        };
        if current != wants
            && let Some(paths) = &self.dictionary_paths
        {
            let path = match wants {
                1 => &paths.jyutping,
                2 => &paths.tungdzih,
                3 => &paths.tupa,
                4 => &paths.shanggu,
                _ => &paths.pinyin,
            };
            match kasm_dictionary::Dictionary::from_path(path) {
                Ok(dictionary) => {
                    engine.replace_dictionary(dictionary);
                    tracing::info!(path = %path.display(), scheme = wants, "主词库已切换");
                }
                Err(error) => {
                    tracing::error!(%error, path = %path.display(), "主词库切换失败，保持原词库")
                }
            }
        }
        engine.set_jyutping(wants == 1);
        engine.set_tungdzih(wants == 2);
        engine.set_tupa(wants == 3);
        engine.set_shanggu(wants == 4);
        // 拼音侧关掉且形码装上了才是「只用形码」；两边都没有时留拼音兜底（否则一个候选都没有）
        engine.set_phonetic(pinyin.is_on() || self.code.is_none());
    }

    fn apply_shuangpin(
        &mut self,
        engine: &mut Engine,
        pinyin: Scheme,
        id: Option<&str>,
        user_dir: Option<&Path>,
        bundled: Option<&Path>,
    ) {
        let wanted = match (pinyin, id) {
            (Scheme::Custom, Some(id)) => {
                let found = shuangpin_schemes::find(id, user_dir, bundled);
                if found.is_none() {
                    tracing::warn!(id, "选了双拼方案但找不到，按全拼输入");
                }
                found
            }
            _ => None,
        };
        if wanted.is_some() && wanted == self.shuangpin {
            return;
        }
        match &wanted {
            Some(entry) => {
                tracing::info!(id = %entry.id, "双拼方案已载入");
                engine.set_shuangpin_layout(Some(Arc::clone(&entry.layout)));
            }
            None => engine.set_shuangpin(pinyin.shuangpin()),
        }
        self.shuangpin = wanted;
    }

    fn apply_code(
        &mut self,
        engine: &mut Engine,
        config: &Config,
        user_dir: Option<&Path>,
        bundled: Option<&Path>,
    ) {
        let entry = config.general.code().and_then(|id| {
            let found = code_schemes::find(&id, user_dir, bundled);
            if found.is_none() {
                tracing::warn!(id, "选了形码方案但找不到，仍按拼音输入");
            }
            found.map(|mut entry| {
                entry
                    .words
                    .retain(|path| is_enabled(&config.dictionaries, path));
                entry.with_rules(config.code_rules(&id))
            })
        });
        let wanted = entry.map(|entry| {
            let stamp = stamp(&entry);
            (entry, stamp)
        });
        if let (Some((entry, stamp)), Some((loaded, loaded_stamp))) = (&wanted, &self.code)
            && same_table(entry, loaded)
            && stamp == loaded_stamp
            && engine.code_scheme().is_some()
        {
            engine.set_code_rules(entry.auto_commit, entry.top);
            self.code = wanted;
            return;
        }
        self.code = wanted.and_then(|(entry, stamp)| match entry.load() {
            Ok(scheme) => {
                tracing::info!(
                    id = %entry.id,
                    table = %entry.table.display(),
                    words = entry.words.len(),
                    entries = scheme.table.len(),
                    "形码方案已载入"
                );
                engine.set_code_scheme(Some(scheme));
                Some((entry, stamp))
            }
            Err(error) => {
                tracing::error!(%error, table = %entry.table.display(), "形码码表读不了，仍按拼音输入");
                None
            }
        });
        if self.code.is_none() {
            engine.set_code_scheme(None);
        }
    }

    /// 装上的用户双拼方案的名字。
    pub fn shuangpin_label(&self) -> Option<&str> {
        self.shuangpin.as_ref().map(|entry| entry.label.as_str())
    }

    /// 装上的形码方案的名字。
    pub fn code_label(&self) -> Option<&str> {
        self.code.as_ref().map(|(entry, _)| entry.label.as_str())
    }

    /// 状态条上的方案名，见 [`scheme_label`]。
    pub fn scheme_label(&self, pinyin: Scheme) -> String {
        scheme_label(pinyin, self.shuangpin_label(), self.code_label())
    }

    /// 主题显示的方案名，见 [`scheme_name`]。
    pub fn scheme_name(&self, pinyin: Scheme) -> String {
        scheme_name(pinyin, self.shuangpin_label(), self.code_label())
    }
}

/// 换了这几项才要重读码表；只改规则不用。
fn same_table(a: &CodeSchemeEntry, b: &CodeSchemeEntry) -> bool {
    a.id == b.id && a.table == b.table && a.words == b.words && a.code_length == b.code_length
}

/// 形码词库与拼音词库共用 `[dictionaries] disabled`，按文件名开关。
fn is_enabled(config: &DictionariesConfig, path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .is_none_or(|stem| config.is_enabled(stem))
}

fn stamp(entry: &CodeSchemeEntry) -> Stamp {
    std::iter::once(&entry.table)
        .chain(&entry.words)
        .map(|path| {
            std::fs::metadata(path).map_or((None, 0), |metadata| {
                (metadata.modified().ok(), metadata.len())
            })
        })
        .collect()
}
