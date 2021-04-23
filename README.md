# A TUI File tree panel
![image](https://user-images.githubusercontent.com/3133596/115800615-4f633880-a3db-11eb-9b08-7509b6c0ec3c.png)


Very simple integration with [kakoune](https://github.com/mawww/kakoune) in tmux:

```
def sidetree %{
  tmux-terminal-impl 'split-window -hb -l 30' kcr shell -s %val[session] -c %val[client] sidetree
}
```

Currently requires [kcr](https://github.com/alexherbo2/kakoune.cr)
