# Ado


## Local Binary
### Install from source

```
 cargo install --path src/bin/ado/
```

### Install from github

```
cargo install --git https://github.com/uintptr/ado
```

### Install `bash` command not found handler

```
    cat <<__EOF__ >> ~/.bashrc
command_not_found_handle(){
    ado --shell-handler "$*"
}
__EOF__

```

### Example

 ![Alt Text](documentation/ado.gif)


## WASM binary

```
wasm-pack build src/lib/adolib/ --target web  -d www/js/pkg
```