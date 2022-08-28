# A TUI File tree panel
![image](https://user-images.githubusercontent.com/3133596/128500582-5682d56c-7f26-4aa1-a3a5-33a0158fcddb.png)

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
map -docstring 'file explorer' global normal <c-e> ':connect panel sidetree --select %val{buffile}<ret>'
```

This also requires [kcr](https://github.com/alexherbo2/kakoune.cr)

## Configuration

Commands can be placed in `~/.config/sidetree/sidetreerc`:
```
set show_hidden false
set quit_on_open false
set open_cmd 'kcr open "${sidetree_entry}"'

set file_icons true
set icon_style darkgray
set dir_name_style lightblue+b
set file_name_style reset
set highlight_style +r
set link_style cyan+b
```

For more examples, see the provided `sidetreerc` file.

## Commands

Commands can be executed by pressing `:` to get the command prompt, or placed in
the config file as described above.

### `quit`
Quit sidetree

### `open [path]`
Open the given path or the currently selected one. See the `open_cmd` option
below for configuration.

### `set <option> <value>`
Set a config option. See options below

### `echo [args...]`
Echo something to the status line. All arguments will be joined by spaces and
echoed.

### `shell [command...]`
Execute `command` in a shell. Without quotes, all arguments are joined by spaces and
executed. 

### `cd [path]`
Change root directory to the given path, or the currently selected folder.

### `map <key> <command> [args...]`
Map a key to another sidetree command. Example: `map H cd ..`

Keys are formatted as one of the following:
 - A character, or one of `return`, `ret`, `semicolon`, `gt`, `lt`, `percent`, `space`,
   `tab`.
 - Any of the above with modifiers `ctrl` or `alt`, formatted as
   `<[mods-...][key]>`, for example `<c-j>`, `<a-j>`, `<c-a-space>` etc.
 - One of the special named keys that do not support modifiers: `esc`, `backtab`,
   `backspace`, `del`, `home`, `end`, `up`, `down`, `left`, `right`, `insert`,
   `pageup`, `pagedown`.

## Expansions

### `sidetree_entry`
Path to selected entry.

### `sidetree_dir`
Path to selected directory.

## Options
Options are set using the `set [option] [value]` command

### `show_hidden: bool`
Whether to show hidden files (file names starting with `.`)

### `open_cmd: String`
The shell command to run to open a file, i.e. on the `:open` command, or when pressing `<return>` on a file. 

Example: `set open_cmd 'xdg-open "$sidetree_entry"'`.

### `quit_on_open: bool`
Whether to quit sidetree after `:open` (or pressing `<return>` on a file)

### `file_icons: bool`
Whether to enable file icons by extension. May or may not be supported by your fonts.

### Styling
Style options have the following format:
`[<fg>][,<bg>][+<add_attr>][-<sub_attr>]`

`<fg>` and `<bg>` are colors, which can either be the name of a color, a
`rgb:XXXXXX` string, where `XXXXXX` is the hex color value, or `colorX`, where
`X` is the index of the terminal color. Valid color names are:
`reset`, `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `gray`,
`darkgray`, `lightred`, `lightgreen`, `lightyellow`, `lightblue`, `lightmagenta`,
`lightcyan`, `white`

`<add_attr>` are attributes to add, and `sub_attr` are attributes to remove.
These are a series of characters, that each correspond to the following
attributes:
```
b → bold
d → dim
i → italic
u → underlined
B → blink
r → reversed
```

Example styles: `blue,black+bu-i`, `,red`, `reset,reset+r`, `-B`

#### `icon_style: Style`: Style of the file/folder icon
#### `dir_name_style: Style`: Style of directory names
#### `file_name_style: Style`: Style of non-directory names
#### `highlight_style: Style`: Style of the highlighted entry
#### `link_style`: Style of symlink names
This style is applied on top of the existing styles, so `+r` could be a good
option, or alternatively `blue,reset+r`.

TODO
----

 - [x] Proper symlink handling
 - [x] Custom key maps, `map` command
 - [x] More navigation commands
 - [x] Save selection & expanded folders between launches
 - [x] Backend for styling entries
 - [x] File icons by extension
 - [x] Custom formatting
 - [x] Better script parsing
   - [x] Comments
   - [x] Blocks
   - [ ] percent-strings
   - [ ] Expansions
 - [x] Auto reflection of configs
 - [ ] Custom commands and aliases
 - [ ] Better prompt keybinds and cursor movements 
 - [ ] Git integration
   - [ ] Gitignore
   - [ ] Git status
 - [ ] IPC for two way syncing
