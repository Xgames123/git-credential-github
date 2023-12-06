# script to verify versions

cargo_ver=$(rg -or "\$1" "^version[\t ]*=[\t ]*\"([0-9]\.[0-9]\.[0-9])\"" Cargo.toml)
echo "version: $cargo_ver"
pkg_ver=$(rg -or "\$1" "^pkgver=[\t ]*([0-9]\.[0-9]\.[0-9])" packaging/PKGBUILD)

git_ver=$(git tag | tail -n 1)


if [ "$cargo_ver" != "$pkg_ver" ] ; then
  echo "PKGBUILD version '$pkg_ver' doesn't match $cargo_ver"
  exit -1
fi

if [ "$git_ver" != "$cargo_ver" ] ; then
  echo "git tag '$git_ver' doesn't match version $cargo_ver"
  exit -1
fi

echo "ALL CHECKS OK"
