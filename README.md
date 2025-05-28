# Ado ( AI DO ? )


## Install from source

```
cargo install --path .
```

## Install from github

```
cargo install --git https://github.com/uintptr/ado
```

## Install `bash` command not found handler

```
    cat <<__EOF__ >> ~/.bashrc
command_not_found_handle(){
    ado --shell-handler "$*"
}
__EOF__

```

## Example

 ![Alt Text](documentation/ado.gif)