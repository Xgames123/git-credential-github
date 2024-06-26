#!/bin/bash
# NOTE: this is designed to run under wsl on windows
os="choco"

#aliases
function cargo() {
  cargo.exe $@
}
function choco() {
  choco.exe $@
}
#end aliases

source ./PKGBUILD

root="$PWD"

echo "TODO: fix me"
srcdir=$PWD/../src
lfjsl
pkgdir=$PWD/pkg_choco/$pkgname

mkdir -p $pkgdir
mkdir -p $pkgdir/tools

echo "Generating nuspec"
echo "<?xml version=\"1.0\" encoding=\"utf-8\"?>
<package xmlns=\"http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd\">
  <metadata>
    <id>$pkgname</id>
    <version>$pkgver</version>
    <packageSourceUrl>$url</packageSourceUrl>
    <owners>$maintainer_short</owners>

    <title>$pkgname</title>
    <authors>$maintainer_short</authors>

    <projectUrl>$url</projectUrl>
    <iconUrl>http://rawcdn.githack.com/Xgames123/$pkgname/$pkgver/logo.png</iconUrl>

    <projectSourceUrl>$url</projectSourceUrl>
    <tags>$tags </tags>
    <summary>$pkgdesc</summary>
    <description>$pkgdesc</description>
  </metadata>
  <files>
    <file src=\"tools\\**\" target=\"tools\" />
  </files>
</package>
" > $pkgdir/$pkgname.nuspec

echo "Copying static choco files..."
cp choco/* $pkgdir/tools

echo "RUNNING prepare()"
prepare

echo "RUNNING build()"
build

echo "RUNNING package()"
package

cd $pkgdir
choco pack
cd $root

mv $pkgdir/$pkgname.$pkgver.nupkg $pkgname.$pkgver.nupkg

