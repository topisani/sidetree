# A TUI File tree panel
![image](https://user-images.githubusercontent.com/3133596/115800615-4f633880-a3db-11eb-9b08-7509b6c0ec3c.png)

Built for integration with [kakoune](https://github.com/mawww/kakoune), but with a little
bit of configuration, it can be used with most TUIs, or even as a minimal
terminal file manager in its own right.

Currently in early alpha stage, PRs are welcome!

## Installation

Since sidetree is published on [crates.io](https://crates.io/crates/sidetree),
any system with `cargo` installed can simply get sidetree by running
```sh
cargo install sidetree
```

## Example integration

Very simple integration with [kakoune](https://github.com/mawww/kakoune) in tmux:

```kak
def sidetree %{
  nop %sh{
    tmux split-window -hb -t 1 -l 30 -e "KAKOUNE_SESSION=$kak_session" -e "KAKOUNE_CLIENT=$kak_client" sidetree -s "$kak_buffile"
  }
}
map -docstring 'sidetree' global user <tab> ': sidetree<ret>'
```

This also requires [kcr](https://github.com/alexherbo2/kakoune.cr)

## Configuration

Commands can be placed in `~/.config/sidetree/sidetreerc`, one command per line:
```
set show_hidden false
set quit_on_open false
set open_cmd 'kcr edit "$sidetree_entry"; kcr send focus'
map <c-c> quit
map H cd ..
map L cd
map / shell kcr send cd "$sidetree_entry"
```

TODO
----

 - [x] Proper symlink handling
 - [x] Custom key maps, `map` command
 - [x] More navigation commands
 - [x] Save selection & expanded folders between launches
 - [x] Backend for styling entries
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
 - [ ] IPC for two way syncing
