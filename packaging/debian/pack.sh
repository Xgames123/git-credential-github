pkgver=0.1.0

pkgname=gh-login
pkgdesc="A simple git credentials helper for github"
license=MIT
url="https://github.com/Xgames123/gh-login"
maintianer=ldev

TARGET=$1
echo "os: Debain"
echo "target: $TARGET"


CARCH=$(echo $TARGET | grep -o "^[^-]*")
builddir=/tmp
fullbuilddir=$builddir/gh-login
binname=git-credential-$pkgname

debarch=$CARCH
if [ "$CARCH" = "x86_64" ]; then
  debarch="amd64"
fi
if [ "$CARCH" = "armv7" ]; then
  debarch="armhf"
fi


rm -rf $fullbuilddir
mkdir -p $fullbuilddir
cp -rf packaging/debian/gh-login $builddir

# Generate control file
controlfile=$fullbuilddir/DEBIAN/control
echo "" > $controlfile
echo "Package: $pkgname" >> $controlfile
echo "Version: $pkgver" >> $controlfile
echo "Maintainer: $maintianer" >> $controlfile
echo "Architecture: $debarch" >> $controlfile
echo "Description: $pkgdesc" >> $controlfile
echo "Homepage: $url" >> $controlfile
echo "Section: vcs" >> $controlfile

# Build rust app
export PKG_CONFIG_SYSROOT_DIR=/

export RUSTUP_TOOLCHAIN=stable
export CARGO_TARGET_DIR=target
cargo build --target $TARGET --release --all-features

export RUSTUP_TOOLCHAIN=stable
cargo test --frozen --all-features

# Copy bin to package
mkdir -p $fullbuilddir/bin
cp -f target/release/$binname $fullbuilddir/bin/$binname
chmod +x $fullbuilddir/bin/$binname

# Building package
dpkg-deb --build $fullbuilddir

# Copy to output dir
mkdir -p target/packaging
mv $builddir/gh-login.deb target/packaging/gh-login-debian-$CARCH.deb