<p align="center">
  <img src="https://github.com/Xgames123/git-credential-github/blob/main/logo.png?raw=true" alt="logo"/>
</p>

A simple git [credentials helper](https://git-scm.com/docs/gitcredentials) for GitHub

![Screenshot of a device code request](example.png)

# Features
* Its way less bloated than [Git Credential Manager](https://github.blog/2022-04-07-git-credential-manager-authentication-for-everyone)
* You can use it together with any other git credential helper of choice

# Install

## Debian/Ubuntu
Download the .deb from the [latest release](https://github.com/xgames123/git-credential-github/releases/latest) and run `dpkg -i file_you_just_downloaded.deb`

## Arch linux
Install git-credential-github form the AUR. [ArchLinux wiki](https://wiki.archlinux.org/title/Arch_User_Repository#Installing_and_upgrading_packages)

# Configuring
NOTE: Configuring changed after v2.2 [pre v2.2 config](PRE_v2_2_CONFIG.md)

```~/.gitconfig```
```ini
[credential "https://github.com"]
  helper = cache
  helper = github # important that you put it last because we only need to run gcg when other helpers have failed to give credentials
```
This sets the credential helper for github using the cache helper with a timeout of 1 day

## More examples

### Set only for repos owned by you
```~/.gitconfig```
```ini
[credential]
	useHttpPath = true # makes git give the whole path instead of just https://github.com
[credential "https://github.com/Xgames123"] # change to your name
  username=Xgames123 # change to your name
  helper = cache
  helper = github
```

### Use pass as the credential helper for everything
```~/.gitconfig```
```ini
[credential]
	useHttpPath = true # makes git give the whole path instead of just https://github.com
    helper = pass -r 3 -t ~/.config/git-credential-pass/default.template -p git/{protocol}/{host}/main

[credential "https://github.com/Xgames123"] # change to your name
  username=Xgames123 # change to your name
  helper = github

[credential "https://codeberg.org"]
  username=ldev
```


# Bug or Error
If you find a bug, get an error or the docs are wrong.
* [Create an issue](https://github.com/Xgames123/git-credential-github/issues/new/)
* Message me <[ldev@ldev.eu.org](mailto://ldev@ldev.eu.org)>
* Message me on discord [ldev105](https://ldev.eu.org/socials/discord)


# Building debian packages from source
1. install [reltools](https://github.com/Xgames123/reltools)
2. run makepkgx --pkgformat deb inside the packaging directory
