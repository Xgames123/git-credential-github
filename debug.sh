alias debug_git=git -c credential.helper="$PWD/target/debug/git-credential-gh-login -b cache -vv"
export RUST_BACKTRACE=1
if [ "$1" = "--path" ] ; then
  export PATH="$PWD/target/debug:$PATH"
  echo "Added debug folder to PATH"
fi


echo "Dev environment setup done"
echo "Use debug_git instead of git to use the debug credential helper"
