TARGET=$1
pkgver=$2

pkgname=gh-login
pkgdesc="A simple git credentials helper for github"
license=MIT
url="https://github.com/Xgames123/gh-login"
maintianer=ldev

echo "os: Debain"
echo "target: $TARGET"
echo "version: $pkgver"

CARCH=$(echo $TARGET | grep -o "^[^-]*")
builddir=/tmp
fullbuilddir=$builddir/gh-login
binname=git-credential-$pkgname

debarch=$CARCH
if [ "$CARCH" = "x86_64" ]; then
  debarch="amd64"
fi
if [ "$CARCH" = "armv7" ]; then
  debarch="arm"
fi


rm -rf $fullbuilddir
mkdir -p $fullbuilddir
#cp -rf packaging/debian/gh-login $builddir

# Generate control file
mkdir $fullbuilddir/DEBIAN
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
export RUSTUP_TOOLCHAIN=stable
cargo fetch --locked --target $TARGET

export RUSTUP_TOOLCHAIN=stable
export CARGO_TARGET_DIR=target
cargo build --frozen --release --all-features

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
