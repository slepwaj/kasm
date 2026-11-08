pluginManagement {
    repositories {
        // 本机直连 Maven Central / Gradle Plugin Portal 不通，先走腾讯公共镜像
        maven("https://mirrors.cloud.tencent.com/nexus/repository/maven-public/")
        google()

    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        maven("https://mirrors.cloud.tencent.com/nexus/repository/maven-public/")
        google()
    }
}

rootProject.name = "kasm-android"
include(":app")
