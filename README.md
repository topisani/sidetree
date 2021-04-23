# A TUI File tree panel

Very simple integration with [kakoune](https://github.com/mawww/kakoune) in tmux:

```
def sidetree %{
  tmux-terminal-impl 'split-window -hb -l 30' kcr shell -s %val[session] -c %val[client] sidetree
}
```

Currently requires [kcr](https://github.com/alexherbo2/kakoune.cr)
