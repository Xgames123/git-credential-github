pkgver=$(cargo run -q -- --version | rg -o '[0-9]\.[0-9]\.[0-9]')

echo Packaging gh-login-debian-armv7
packaging/debian/pack.sh armv7-unknown-linux-gnueabihf $pkgver
echo Packaging gh-login-debian-x86_64
packaging/debian/pack.sh x86_64-unknown-linux-gnu $pkgver

echo "Done"
echo "Packages at target/packaging"
