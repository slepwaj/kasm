//! 鸿蒙 PoC 的引擎门面：C ABI，给 NAPI 的 C++ 薄封装调用。
//!
//! 现在只覆盖最小链路：建引擎、按键组句、取候选、空格上屏、退格。
//! 数据由 ArkTS 侧把随包词库解到沙箱后把路径传进来。

use std::ffi::{CStr, CString, c_char, c_void};
use std::path::PathBuf;
use std::ptr;

use kasm_core::Engine;
use kasm_dictionary::Dictionary;
use serde_json::json;

/// 一个输入会话：组句串、当前候选、preedit。
pub struct Session {
    engine: Engine,
    composition: String,
    candidates: Vec<String>,
    preedit: String,
    /// 主词库目录（`dict.tsv` 所在）；方案词库在旁边的 `lexicon/`。
    data_dir: PathBuf,
    base_dict: PathBuf,
}

fn frame(session: &Session) -> String {
    json!({
        "preedit": session.preedit,
        "candidates": session.candidates,
    })
    .to_string()
}

fn to_c_string(text: String) -> *mut c_char {
    CString::new(text)
        .map(CString::into_raw)
        .unwrap_or(ptr::null_mut())
}

impl Session {
    fn new(dict_path: &str) -> Option<Self> {
        let dictionary = Dictionary::from_path(std::path::Path::new(dict_path)).ok()?;
        let base_dict = std::path::PathBuf::from(dict_path);
        let data_dir = base_dict.parent()?.to_path_buf();
        Some(Self {
            engine: Engine::new(dictionary),
            composition: String::new(),
            candidates: Vec::new(),
            preedit: String::new(),
            data_dir,
            base_dict,
        })
    }

    fn refresh(&mut self) {
        if self.composition.is_empty() {
            self.candidates.clear();
            self.preedit.clear();
            return;
        }
        self.engine.set_input(&self.composition);
        match self.engine.query() {
            Ok(query) => {
                self.candidates = query
                    .candidates
                    .items
                    .iter()
                    .take(30)
                    .map(|candidate| candidate.text.clone())
                    .collect();
                self.preedit = query.marked_text();
            }
            Err(_) => {
                self.candidates.clear();
                self.preedit = self.composition.clone();
            }
        }
    }

    /// 切拼音方案：`""` 全拼，或 jyutping / tungdzih / tupa / shanggu。换主词库并让开双拼/五笔。
    fn set_scheme(&mut self, name: &str) -> bool {
        let known = matches!(name, "jyutping" | "tungdzih" | "tupa" | "shanggu");
        let path = if known {
            self.data_dir.join("lexicon").join(format!("{name}.tsv"))
        } else {
            self.base_dict.clone()
        };
        let Ok(dictionary) = Dictionary::from_path(&path) else {
            return false;
        };
        self.engine.replace_dictionary(dictionary);
        self.engine.set_jyutping(name == "jyutping");
        self.engine.set_tungdzih(name == "tungdzih");
        self.engine.set_tupa(name == "tupa");
        self.engine.set_shanggu(name == "shanggu");
        if known {
            self.engine.set_shuangpin(None);
            self.engine.set_code_scheme(None);
            self.engine.set_phonetic(true);
        }
        self.composition.clear();
        self.candidates.clear();
        self.preedit.clear();
        true
    }

    fn key(&mut self, ch: char) {
        if ch.is_ascii_alphabetic() {
            self.composition.push(ch.to_ascii_lowercase());
            self.refresh();
        }
    }

    fn backspace(&mut self) {
        self.composition.pop();
        self.refresh();
    }

    fn commit(&mut self, index: usize) -> Option<String> {
        let text = self.candidates.get(index).cloned()?;
        self.composition.clear();
        self.candidates.clear();
        self.preedit.clear();
        Some(text)
    }

    fn commit_first(&mut self) -> Option<String> {
        self.commit(0)
    }
}

/// 建会话；`dict_path` 是 UTF-8 的 C 字符串。失败返回空指针。
#[unsafe(no_mangle)]
pub extern "C" fn kasm_create(dict_path: *const c_char) -> *mut c_void {
    if dict_path.is_null() {
        return ptr::null_mut();
    }
    let Ok(path) = unsafe { CStr::from_ptr(dict_path) }.to_str() else {
        return ptr::null_mut();
    };
    match Session::new(path) {
        Some(session) => Box::into_raw(Box::new(session)).cast(),
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kasm_key(handle: *mut c_void, ch: u32) -> *mut c_char {
    let Some(session) = (unsafe { (handle as *mut Session).as_mut() }) else {
        return ptr::null_mut();
    };
    if let Some(ch) = char::from_u32(ch) {
        session.key(ch);
    }
    to_c_string(frame(session))
}

#[unsafe(no_mangle)]
pub extern "C" fn kasm_backspace(handle: *mut c_void) -> *mut c_char {
    let Some(session) = (unsafe { (handle as *mut Session).as_mut() }) else {
        return ptr::null_mut();
    };
    session.backspace();
    to_c_string(frame(session))
}

#[unsafe(no_mangle)]
pub extern "C" fn kasm_space(handle: *mut c_void) -> *mut c_char {
    let Some(session) = (unsafe { (handle as *mut Session).as_mut() }) else {
        return ptr::null_mut();
    };
    let commit = session.commit_first();
    to_c_string(
        json!({
            "commit": commit,
            "preedit": "",
            "candidates": [],
        })
        .to_string(),
    )
}

/// 选中第 `index` 个候选并上屏；返回 `{"commit": "..."}`。
#[unsafe(no_mangle)]
pub extern "C" fn kasm_select(handle: *mut c_void, index: u32) -> *mut c_char {
    let Some(session) = (unsafe { (handle as *mut Session).as_mut() }) else {
        return ptr::null_mut();
    };
    let commit = session.commit(index as usize);
    to_c_string(
        json!({
            "commit": commit,
            "preedit": "",
            "candidates": [],
        })
        .to_string(),
    )
}

/// 切拼音方案；返回 1 成功、0 失败。
#[unsafe(no_mangle)]
pub extern "C" fn kasm_set_scheme(handle: *mut c_void, name: *const c_char) -> i32 {
    let Some(session) = (unsafe { (handle as *mut Session).as_mut() }) else {
        return 0;
    };
    if name.is_null() {
        return 0;
    }
    let Ok(name) = (unsafe { CStr::from_ptr(name) }).to_str() else {
        return 0;
    };
    i32::from(session.set_scheme(name))
}

#[unsafe(no_mangle)]
pub extern "C" fn kasm_destroy(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(handle as *mut Session));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kasm_free_string(text: *mut c_char) {
    if text.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(text));
    }
}
