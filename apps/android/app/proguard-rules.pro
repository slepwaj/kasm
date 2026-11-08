# JNI 入口与 Core 包装类不能被 R8 改名，否则 native 方法找不到
-keep class app.kasm.inputmethod.core.Native { *; }
-keep class app.kasm.inputmethod.core.Engine { *; }
-keep class app.kasm.inputmethod.core.TranslationEngine { *; }
-keepclasseswithmembers class * { native <methods>; }
-keep class app.kasm.inputmethod.core.frame.** { *; }
