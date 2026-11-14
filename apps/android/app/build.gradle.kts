plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.android)
}

// 仓库根（ime/）：Rust workspace 与随包数据都从这里取。
val repoRoot = rootDir.resolve("../..").canonicalFile
val nativeLibs = layout.buildDirectory.dir("native/jniLibs")
val bundledAssets = layout.buildDirectory.dir("bundled/assets")

// 版本号取 core/Cargo.toml；versionCode = 主*10000 + 次*100 + 修订，-dev 与正式版同号
val appVersion = rootDir.resolve("core/Cargo.toml").readLines()
    .first { it.startsWith("version = ") }
    .substringAfter('"').substringBefore('"')
val appVersionCode = appVersion.substringBefore('-').split('.').map(String::toInt)
    .let { (major, minor, patch) -> major * 10000 + minor * 100 + patch }

// 发版 CI 从 Secrets 注入，见 docs/notes/release.md
val releaseKeystore = System.getenv("KASM_ANDROID_KEYSTORE")?.takeIf { it.isNotEmpty() }

android {
    namespace = "app.kasm.inputmethod"
    compileSdk = 36
    ndkVersion = "29.0.13599879"

    defaultConfig {
        applicationId = "app.kasm.inputmethod"
        minSdk = 26
        targetSdk = 36
        versionCode = appVersionCode
        versionName = appVersion
        ndk { abiFilters += "arm64-v8a" }
        manifestPlaceholders["appLabel"] = "KASM"
    }

    if (releaseKeystore != null) {
        signingConfigs {
            create("release") {
                storeFile = file(releaseKeystore)
                storePassword = System.getenv("KASM_ANDROID_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("KASM_ANDROID_KEY_ALIAS")
                keyPassword = System.getenv("KASM_ANDROID_KEY_PASSWORD")
            }
        }
    }

    buildFeatures {
        buildConfig = true
    }

    buildTypes {
        debug {
            // 手机上装着发版签名的包时并排装调试版（-PsideBySide），不用卸载、不丢数据
            if (project.hasProperty("sideBySide")) applicationIdSuffix = ".dev"
            // 与正式版并存时，系统设置和输入法列表里能分清两个 KASM
            manifestPlaceholders["appLabel"] = "KASM 调试版"
        }
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
            if (releaseKeystore != null) signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets["main"].apply {
        jniLibs.srcDir(nativeLibs)
        assets.srcDir(bundledAssets)
    }
}

kotlin {
    compilerOptions {
        jvmTarget = org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17
    }
}

// Rust 侧：cargo ndk 编出 libkasm_android.so，放进 jniLibs 源目录。要先 `cargo install cargo-ndk`。
val cargoNdk by tasks.registering(Exec::class) {
    group = "kasm"
    description = "cargo ndk 编译 kasm-android（arm64-v8a，release）"
    workingDir = repoRoot
    val ndkDir = android.sdkDirectory.resolve("ndk/${android.ndkVersion}")
    environment("ANDROID_NDK_HOME", ndkDir.absolutePath)
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "--platform", "26",
        "-o", nativeLibs.get().asFile.absolutePath,
        "build", "-p", "kasm-android", "--release", "--locked",
    )
    // 增量交给 cargo 判断，这里每次都跑
    outputs.upToDateWhen { false }
}

// 随包数据：data/generated/ 的产品数据优先，没有就退回 assets/sample/ 样例（与 mac 的 bundle.sh 同一规矩）。
val stageData by tasks.registering(Sync::class) {
    group = "kasm"
    description = "把随包数据放进 assets/data/"
    val generated = repoRoot.resolve("data/generated")
    val hasGenerated = generated.resolve("dict.kasm").isFile
    into(bundledAssets.map { it.dir("data") })
    if (hasGenerated) {
        from(generated) {
            // 学习语言可选，几门语言的释义表都带上（共约 65 MB）
            include("dict.kasm", "lm.kasm", "glossary-*.kasm", "english.tsv")
        }
        // 随包领域词库（与 PC 同一份），词库页里勾选
        from(generated.resolve("dicts")) { into("dicts") }
    } else {
        // 没有打包好的产品词库时用仓库里的完整词库（13 万条）；assets/sample/dict.tsv 只有几百条，
        // 拿它打出来的包全拼几乎没候选。
        from(repoRoot.resolve("assets/lexicon")) { include("dict.tsv") }
        from(repoRoot.resolve("assets/sample")) { include("english.tsv") }
        // 学习语言的释义表用完整版（assets/sample 里只有 2 KB 样例，候选旁不会出译词）
        from(repoRoot.resolve("assets/glossary")) { include("glossary-*.tsv") }
    }
    from(repoRoot.resolve("assets/emoji")) { include("emoji-zh.tsv", "emoji-en.tsv") }
    // 打字练习文段：随机抽句组成练习段
    from(repoRoot.resolve("assets/practice")) {
        include("sentences.txt", "readings.tsv")
        into("practice")
    }
    // 粤拼 / 通字 / tupa / 上古音：安卓的「拼音方案」选择用
    from(repoRoot.resolve("assets/lexicon")) {
        include("jyutping.tsv", "tungdzih.tsv", "tupa.tsv", "shanggu.tsv", "unt.tsv", "gcr.tsv", "xina.tsv")
        into("lexicon")
    }
    // 词汇等级表（英语 CEFR、日语 JLPT），生词本按级筛与统计页按级分用
    from(repoRoot.resolve("assets/levels")) { include("levels-*.tsv") }
    // 五笔码表（形码方案用，与 PC 随包的同一份），解出来仍叫 wubi86.tsv
    from(repoRoot.resolve("assets/schemes/wubi86")) {
        include("table.tsv")
        rename { "wubi86.tsv" }
    }
}

tasks.named("preBuild") {
    dependsOn(cargoNdk, stageData)
}
