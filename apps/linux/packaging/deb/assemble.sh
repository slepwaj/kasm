#!/usr/bin/env bash
# 在 Debian 13 容器里跑（build.sh 调，仓库挂在 /src）：编 fcitx5 插件，从二进制推出依赖，拼两个 .deb：
# - kasm：Server、IBus 前端、词库、systemd 用户服务（build.sh 已摆好在 $stage/kasm）
# - kasm-fcitx5：fcitx5 插件
set -euo pipefail
stage=/src/target/linux-pkg/deb
version=${DEB_VERSION:?}
arch=$(dpkg --print-architecture)
multiarch=$(dpkg-architecture -qDEB_HOST_MULTIARCH)
maintainer='Kasm <https://kasm.app>'

build="$stage/fcitx5-build"
cmake -S /src/apps/linux/fcitx5 -B "$build" -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=OFF >/dev/null
cmake --build "$build" --parallel >/dev/null
fcitx="$stage/kasm-fcitx5"
# 插件放 fcitx5 自己的插件目录（Debian 是多架构路径），描述文件里 Library=kasm 按名字找
install -Dm644 "$build/kasm.so" "$fcitx/usr/lib/$multiarch/fcitx5/kasm.so"
strip --strip-unneeded "$fcitx/usr/lib/$multiarch/fcitx5/kasm.so"
install -Dm644 /src/apps/linux/fcitx5/data/addon/kasm.conf "$fcitx/usr/share/fcitx5/addon/kasm.conf"
install -Dm644 /src/apps/linux/fcitx5/data/inputmethod/kasm.conf "$fcitx/usr/share/fcitx5/inputmethod/kasm.conf"
install -Dm644 /src/LICENSE "$fcitx/usr/share/doc/kasm-fcitx5/copyright"

core="$stage/kasm"
strip --strip-unneeded "$core/usr/bin/kasm-linux-server" "$core/usr/bin/kasm-linux-ibus"

# 动态库依赖从二进制推（libc6 的最低版本、libfcitx5core7 这些），不手写；
# dpkg-shlibdeps 只肯在带 debian/control 的目录里跑，给它一个最小的
shlibs() {
  local dir
  dir=$(mktemp -d)
  mkdir "$dir/debian"
  printf 'Source: kasm\n\nPackage: kasm\nArchitecture: any\n' > "$dir/debian/control"
  (cd "$dir" && dpkg-shlibdeps -O "$@" | sed -n 's/^shlibs:Depends=//p')
  rm -rf "$dir"
}

# 权限归一：宿主机 umask 是 002 时拷出来的是 664 / 775
normalize() {
  find "$1" -type d -exec chmod 755 {} +
  find "$1" -type f ! -path '*/usr/bin/*' -exec chmod 644 {} +
}

control() {
  local dir=$1 name=$2 depends=$3 extra=$4 summary=$5 description=$6
  mkdir -p "$dir/DEBIAN"
  cat > "$dir/DEBIAN/control" <<CONTROL
Package: $name
Version: $version
Architecture: $arch
Maintainer: $maintainer
Installed-Size: $(du -sk --exclude=DEBIAN "$dir" | cut -f1)
Depends: $depends
${extra}Section: utils
Priority: optional
Homepage: https://kasm.app
Description: $summary
$description
CONTROL
}

normalize "$core"
control "$core" kasm "$(shlibs "$core/usr/bin/kasm-linux-server" "$core/usr/bin/kasm-linux-ibus")" \
  $'Recommends: ibus | kasm-fcitx5\n' \
  'KASM输入法：Server、IBus 前端与词库' \
  ' 跨平台拼音输入法。本包是引擎（Server，systemd 用户服务）、IBus 前端与随包词库；
 用 fcitx5 的再装 kasm-fcitx5。Server 装完就起；IBus 要重新登录一次（或 ibus restart）才看得到KASM。'
# Server 挂到每个用户的会话里自启。--global 只对之后起来的用户 systemd 实例生效，而注销再登录得快时
# 实例根本不重起（Ubuntu 24.04 上踩过：装完重登，Server 一次都没起）；所以已经登录的用户直接在他们
# 正在跑的实例里重载并（重）启动：装完不用重登就能打字，升级也换上新的 Server
cat > "$core/DEBIAN/postinst" <<'SCRIPT'
#!/bin/sh
set -e
if [ "$1" = configure ] && command -v systemctl >/dev/null 2>&1; then
  systemctl --global enable kasm-server.service >/dev/null 2>&1 || true
  # 用户目录里的同名单元（源码安装的 install.sh 登记的）会盖过这个包的 Server：
  # 前端用这个包的、Server 用那边的，版本一错开就打不出字（本机踩过：Server 断开会话，按键全部放行）
  getent passwd | while IFS=: read -r name _ uid _ _ home _; do
    [ "$uid" -ge 1000 ] 2>/dev/null || continue
    if [ -f "$home/.config/systemd/user/kasm-server.service" ]; then
      # 拆成短行：dnf5 按终端宽度截断装包脚本的输出
      echo "注意：用户 $name 有自己的KASM Server 服务，会盖过这个包装的：" >&2
      echo "  $home/.config/systemd/user/kasm-server.service" >&2
      echo "  以该用户卸掉那一份——源码安装的：apps/linux/scripts/uninstall.sh" >&2
    fi
  done
  loginctl list-users --no-legend 2>/dev/null | while read -r _ user _; do
    systemctl --user -M "$user@" daemon-reload >/dev/null 2>&1 || true
    systemctl --user -M "$user@" restart kasm-server.service >/dev/null 2>&1 || true
  done
fi
SCRIPT
cat > "$core/DEBIAN/prerm" <<'SCRIPT'
#!/bin/sh
set -e
if [ "$1" = remove ] && command -v systemctl >/dev/null 2>&1; then
  systemctl --global disable kasm-server.service >/dev/null 2>&1 || true
  loginctl list-users --no-legend 2>/dev/null | while read -r _ user _; do
    systemctl --user -M "$user@" stop kasm-server.service >/dev/null 2>&1 || true
  done
fi
SCRIPT
chmod 755 "$core/DEBIAN/postinst" "$core/DEBIAN/prerm"

normalize "$fcitx"
control "$fcitx" kasm-fcitx5 "kasm (= $version), fcitx5, $(shlibs "$fcitx/usr/lib/$multiarch/fcitx5/kasm.so")" '' \
  'KASM输入法的 fcitx5 插件' \
  ' 让 fcitx5 能用KASM；引擎在 kasm 包里。装完重启 fcitx5，在配置工具里添加「KASM」。'

for dir in "$core" "$fcitx"; do
  dpkg-deb --root-owner-group -Zxz --build "$dir" "$stage/$(basename "$dir")_${version}_${arch}.deb" >/dev/null
done
ls "$stage"/*.deb
