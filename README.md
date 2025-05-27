# Build + Install

```
cargo install --path .
```

# Bash command not found handler

```
    cat <<__EOF__ >> ~/.bashrc
command_not_found_handle(){
    ado --shell-handler "$*"
}
__EOF__

```