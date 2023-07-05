<p align="center">
  <img src="https://github.com/Xgames123/gh-login/blob/main/gh-login-logo_200px_transparent.png?raw=true" alt="gh-login-logo"/>
</p>
A simple git credentials manager for GitHub

It authenticates to GitHub and uses a backing credential helper, so you can use normal git credential helpers.

## TODO
* Make the 'https://github.com/login/device' link a real link
* Make an enviroment variable to set the backing helper. eg GHLOGIN_BACKINGHELPER
* Support Archlinux (aur)
* Support Windows

## Install
1. Install gh-login for your os (see below)
2. Set gh-login as your git credential helper
   
    ```git config --global credential.https://github.com.helper 'gh-login -b cache'```
   
    You can change cache to any credential helper you like

    If that gives an error you can also manually edit $HOME/.gitconfig

### Arch linux
Install gh-login form the AUR
```bash
git clone aur.archlinux.org/gh-login.git
cd gh-login
makepkg --syncdeps --install
```

### Debian/Ubuntu
Download latest release and run ```dpkg -i gh-login.deb```
Replace ```gh-login.deb``` with the file you just downloaded


## Bug or Error
If you find a bug, get an error or the docs are wrong.
* [Create an issue](https://github.com/Xgames123/gh-login/issues/new/)
* Message me <[ldev@ldev.eu.org](mailto://ldev@ldev.eu.org)>
* Message me on discord [ldev105](https://ldev.eu.org/socials/discord)
