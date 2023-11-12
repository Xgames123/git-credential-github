# A crappy makepkg clone to build for debian
# Usage makepkg_deb
os="deb"

source ./PKGBUILD


srcdir=$PWD/src
pkgdir=$PWD/pkg/$pkgname

mkdir -p $srcdir
mkdir -p $pkgdir
mkdir -p $pkgdir/DEBIAN

echo "Downloading sources..."
for x_srcfile in "${source[@]}" ; do
  file_name=${x_srcfile%::*}
  download_url=${x_srcfile#*::}
  echo ${download_url//[$'\t\r\n ']}
  
  http_code=$(curl -L -s -o $file_name -w "%{http_code}" ${download_url//[$'\t\r\n ']})
  if [ "$http_code" == "404" ] ; then
    echo "Failed to download sources. status: $http_code"
    exit 1
  fi
  tar -xf $file_name -C $srcdir
done

debarch=$arch
if [ "$arch" = "x86_64" ]; then
  debarch="amd64"
fi
if [ "$arch" = "armv7h" ]; then
  debarch="armhf"
fi

x_debdepends=""
for x_deb in "${depends[@]}" ; do
  x_debdepends="$x_deb, $x_debdepends"
done
x_debdepends="${x_debdepends::-2}"

x_debconflicts=""
for x_deb in "${conflicts[@]}" ; do
  x_debconflicts="$x_deb, $x_debconflicts"
done
x_debconflicts="${x_debconflicts::-2}"

x_debprovides=""
for x_deb in "${provides[@]}" ; do
  x_debprovides="$x_deb, $x_debprovides"
done
x_debprovide="${x_debprovides::-2}"

x_controlfile=$pkgdir/DEBIAN/control
if [ -f $x_controlfile ] ; then
  rm -f $x_controlfile
fi
x_controlfile="${x_controlfile#, }"

x_required ()
{
  if [ "$2" = "" ] ; then
    echo "FAIL: $3 is required for deb packages"
  fi
  echo "$1: $2" >> $x_controlfile
}
x_optional ()
{
  if [ "$2" != "" ] ; then
    echo "$1: $2" >> $x_controlfile
  fi
}

x_required "Package" "$pkgname" "pkgname"
x_required "Architecture" "$debarch" "arch"
x_required "Description" "$pkgdesc" "pkgdesc"
x_required "Version" "$pkgver" "pkgver"

x_optional "Section" "$section" "section"
x_optional "Maintainer" "$maintainer" "maintainer"
x_optional "Homepage" "$url" "url"

x_optional "Depends" "$x_debdepends" "depends"
x_optional "Conflicts" "$x_debconflicts" "conflicts"
x_optional "Provides" "$x_debprovides" "provides"


x_cwd=$PWD

echo "RUNNING prepare()"
prepare

echo "RUNNING build()"
build

echo "RUNNING package()"
package

cd $x_cwd
dpkg --build $pkgdir
mv $pkgdir/../$pkgname.deb $pkgname-$pkgver.deb
