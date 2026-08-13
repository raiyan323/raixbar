pkgname=raixbar
pkgver=0.1.0
pkgrel=1
pkgdesc="Fast and minimal Wayland application launcher"
arch=('x86_64')
url="https://github.com/raiyan323/raixbar"
license=('MIT')

depends=('wayland')
makedepends=('cargo' 'git')

source=("git+https://github.com/raiyan323/raixbar.git")
sha256sums=('SKIP')

prepare() {
    cd "$srcdir/raixbar"
    export CARGO_HOME="$srcdir/cargo"
    cargo fetch
}

build() {
    cd "$srcdir/raixbar"
    export CARGO_HOME="$srcdir/cargo"
    cargo build --release
}

package() {
    cd "$srcdir/raixbar"

    install -Dm755 \
        "target/release/raixbar" \
        "$pkgdir/usr/bin/raixbar"
}