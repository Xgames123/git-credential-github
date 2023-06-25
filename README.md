<p align="center">
  <img src="https://github.com/Xgames123/gh-login/blob/main/gh-login-logo_200px_transparent.png?raw=true" alt="gh-login-logo"/>
</p>
A simple git credentials manager for GitHub

It authenticates to GitHub and uses a backing credential helper, so you can use normal git credential helpers.

## TODO
* Support Archlinux (aur)
* Support Windows

## Install
1. Install gh-login for your os (see below)
2. Set gh-login as your git credential helper
   
    ```git config --global credential.https://github.com.helper 'gh-login -b cache'```
   
    You can change cache to any credential helper you like

    If that gives an error you can also manually edit $HOME/.gitconfig

### Debian/Ubuntu
Download latest release and run ```dpkg -i gh-login.deb```
Replace ```gh-login.deb``` with the file you just downloaded