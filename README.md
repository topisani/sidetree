# A TUI File tree panel
![image](https://user-images.githubusercontent.com/3133596/115800615-4f633880-a3db-11eb-9b08-7509b6c0ec3c.png)


Very simple integration with [kakoune](https://github.com/mawww/kakoune) in tmux:

```
def sidetree %{
  tmux-terminal-impl 'split-window -hb -l 30' kcr shell -s %val[session] -c %val[client] sidetree
}
map -docstring 'sidetree' global user <tab> ': sidetree<ret>'
```

Currently requires [kcr](https://github.com/alexherbo2/kakoune.cr)

## Configuration

Commands can be placed in `~/.config/sidetree/sidetreerc`, one command per line:
```
set show_hidden false
set quit_on_open false
set open_cmd 'kcr edit "$sidetree_entry"; kcr focus'
```

TODO
----

 - [ ] Proper symlink handling
 - [x] Custom key maps, `map` command
 - [ ] Custom commands and aliases
 - [ ] Better script parsing
   - [ ] Comments
   - [ ] Blocks
 - [ ] Custom formatting
 - [ ] Git integration
   - [ ] Gitignore
   - [ ] Git status
 - [ ] File icons by extension
 - [ ] Better prompt keybinds and cursor movements 
