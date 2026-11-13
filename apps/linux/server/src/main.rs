//! Linux 本地输入服务；云联想与本地模型按配置加载，之后改配置由热加载应用。
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod paths;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use kasm_core::Language;
    use kasm_dictionary::Dictionary;
    use kasm_linux_server::{
        AssemblySpec, LanguageModelFiles, Router, RouterConfig, assembly, find_model,
    };
    use kasm_platform::Config;
    use std::collections::HashMap;
    use std::path::PathBuf;
    if std::env::args().any(|arg| arg == "--version") {
        println!("kasm-linux-server {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let config_path = paths::config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Config::write_template_if_missing(&config_path)?;
    let config = Config::load(&config_path)?;
    std::fs::create_dir_all(paths::log_dir())?;
    let appender = tracing_appender::rolling::RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("server")
        .filename_suffix("log")
        .max_log_files(7)
        .build(paths::log_dir())?;
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_ansi(false)
        .with_writer(writer)
        .init();
    let root = paths::resource_root().ok_or("product resources missing; set KASM_RESOURCES")?;
    // 粤拼用粤语词库；其余方案用普通话词库
    let jyutping = config.general.scheme() == kasm_platform::Scheme::Jyutping;
    let tungdzih = config.general.scheme() == kasm_platform::Scheme::Tungdzih;
    let tupa = config.general.scheme() == kasm_platform::Scheme::Tupa;
    let shanggu = config.general.scheme() == kasm_platform::Scheme::Shanggu;
    let dictionary = std::env::var_os("KASM_DICT")
        .map(PathBuf::from)
        .or_else(|| {
            if jyutping {
                paths::generated(&root, "jyutping.kasm")
                    .or_else(|| paths::asset(&root, "lexicon/jyutping.tsv"))
            } else if tungdzih {
                paths::generated(&root, "tungdzih.kasm")
                    .or_else(|| paths::asset(&root, "lexicon/tungdzih.tsv"))
            } else if tupa {
                paths::generated(&root, "tupa.kasm")
                    .or_else(|| paths::asset(&root, "lexicon/tupa.tsv"))
            } else if shanggu {
                paths::generated(&root, "shanggu.kasm")
                    .or_else(|| paths::asset(&root, "lexicon/shanggu.tsv"))
            } else {
                paths::generated(&root, "dict.kasm")
            }
        })
        .unwrap_or_else(|| root.join("assets/sample/dict.tsv"));
    let user_dir = paths::user_dir();
    let mut spec = AssemblySpec {
        glossary: assembly::learning_language(&config)
            .and_then(|lang| assembly::glossary_file(&root, lang).map(|p| (lang, p))),
        english_glossary: assembly::glossary_file(&root, Language::Chinese),
        english: paths::generated(&root, "english.tsv")
            .or_else(|| paths::asset(&root, "sample/english.tsv")),
        emoji: ["emoji/emoji-zh.tsv", "emoji/emoji-en.tsv"]
            .into_iter()
            .filter_map(|n| paths::asset(&root, n))
            .collect(),
        language_model: LanguageModelFiles::find(&root.join("data/generated")),
        bundled_dicts_dir: Some(root.join("data/generated/dicts")),
        dictionaries: config.dictionaries.clone(),
        user_dir: Some(user_dir.clone()),
        input_log: config.general.input_log,
        log_dir: Some(paths::log_dir()),
        ..AssemblySpec::new(dictionary)
    };
    let mut engine = match assembly::assemble(&spec) {
        Ok(engine) => engine,
        Err(error) => {
            tracing::warn!(%error, "主词库不可用，降级样例词库");
            spec.dict = root.join("assets/sample/dict.tsv");
            assembly::assemble(&spec)?
        }
    };
    engine.set_jyutping(jyutping);
    engine.set_tungdzih(tungdzih);
    engine.set_tupa(tupa);
    engine.set_shanggu(shanggu);
    // 通字：繁体词库与简繁对照一起装；set_traditional 负责按输出字形整份换。
    if tungdzih {
        if let Some(path) = paths::generated(&root, "tungdzih-traditional.kasm")
            .or_else(|| paths::asset(&root, "lexicon/tungdzih-traditional.tsv"))
        {
            match Dictionary::from_path(&path) {
                Ok(dictionary) => {
                    tracing::info!(path = %path.display(), entries = dictionary.len(), "通字繁体词库已加载");
                    engine.set_traditional_dictionary(dictionary);
                }
                Err(error) => {
                    tracing::warn!(%error, path = %path.display(), "通字繁体词库加载失败")
                }
            }
        }
        if let Some(path) = paths::asset(&root, "lexicon/tungdzih-script.tsv") {
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    let mut map = HashMap::new();
                    for line in text.lines() {
                        if line.starts_with('#') || line.trim().is_empty() {
                            continue;
                        }
                        if let Some((traditional, simplified)) = line.split_once('\t') {
                            map.insert(traditional.to_owned(), simplified.to_owned());
                        }
                    }
                    engine.set_traditional_seed(map);
                }
                Err(error) => {
                    tracing::warn!(%error, path = %path.display(), "通字简繁对照加载失败")
                }
            }
        }
    }
    engine.set_fuzzy(config.fuzzy);
    engine.set_shuangpin(config.general.shuangpin());
    engine.set_shuangpin_raw_preedit(config.general.shuangpin_raw_preedit);
    engine.set_shuangpin_full_pinyin(config.general.shuangpin_full_pinyin);
    engine.set_hold_selections(config.general.hold_selections);
    engine.set_context_memory(config.model.context_memory_seconds);
    engine.set_shift_letter_compose(config.general.shift_letter.compose());
    engine.set_learning(config.general.learning);
    engine.set_chinese_first(config.general.chinese_first);
    engine.set_spelling_correction(config.general.spelling_correction);
    engine.set_emoji_limit(usize::from(config.general.emoji.get()));
    engine.set_kaomoji_limit(usize::from(config.general.kaomoji.get()));
    engine.set_symbol_limit(usize::from(config.general.symbols.get()));
    engine.set_traditional(config.general.traditional());
    engine.set_english_in_chinese(config.general.english_in_chinese);
    engine.set_english_max_in_chinese(config.general.english_max_in_chinese());
    engine.set_english_min_position(config.general.english_min_position());
    engine.set_mode_keys(config.shortcut.mode);
    engine
        .set_custom_phrases(config.custom_phrases.clone())
        .map_err(std::io::Error::other)?;
    let usage_path = user_dir.join(kasm_predict::API_USAGE_FILE);
    assembly::attach_cloud(&mut engine, &config.predict, Some(usage_path));
    engine.log_session(env!("CARGO_PKG_VERSION"), "linux");
    let mut router = Router::new(engine, RouterConfig::from(&config));
    router.configure_local_model(find_model(Some(&user_dir), &root), &config.model);
    router.watch_config(&config, config_path, root, user_dir);
    extern "C" fn stop(_: libc::c_int) {
        kasm_linux_server::ipc::request_shutdown();
    }
    unsafe {
        libc::signal(libc::SIGTERM, stop as *const () as libc::sighandler_t);
        libc::signal(libc::SIGINT, stop as *const () as libc::sighandler_t);
    }
    let socket = kasm_platform::protocol::linux::socket_path();
    tracing::info!(path = %socket.display(), "KASM Linux Server 启动");
    kasm_linux_server::ipc::serve_socket(socket, &mut router)?;
    Ok(())
}
fn main() {
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    if let Err(error) = run() {
        eprintln!("KASM启动失败：{error}");
        std::process::exit(1);
    }
    #[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
    eprintln!("kasm-linux-server 仅支持 Linux 与 FreeBSD");
}
