# Configuring
```~/.gitconfig```
```ini
[credential "https://github.com"]
  helper = github -b 'cache --timeout=86400'
```
This sets the credential helper for github using the cache helper with a timeout of 1 day

## More examples

### Set only for repos owned by you
```~/.gitconfig```
```ini
[credential]
	useHttpPath = true
[credential "https://github.com/Xgames123"] # change to your name
  username=Xgames123 # change to your name
  helper = github -b 'cache --timeout=86400'
```

### Using pass to store your credentials
Install [git-credential-pass](https://github.com/Xgames123/git-credential-pass)
```~/.gitconfig```
```ini
[credential "https://github.com"]
  helper = github -b 'pass -p git/{host}/{username} -t ~/.config/git-credential-pass/default.template'
```
See [git-credential-pass](https://github.com/Xgames123/git-credential-pass) for more info
