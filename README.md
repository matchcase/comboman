# comboman - Shell Command Selector and Sequencer
`comboman` lets you select a number of commands from your recent shell history, and quickly turn them into a shell script, shell function or a quick access `combo`. It makes repeated actions, such as a sequence of steps for mounting and `cd`ing into a drive, much easier to save and do again, without requiring the user to begin recording the action beforehand. 

## Installation
Run `cargo build --release` to get a `comboman` executable, which you can move to a directory in your `PATH`.

## Usage
`comboman` comes with a set of sub-commands to help use and manage combos.
- `comboman list` lists existing combos
- `comboman delete <combo_name>` deletes the combo `<combo_name>`
- `comboman` or `comboman run` opens a fuzzy menu for selecting a combo to run
You can also use `comboman run <combo_name>` to run a specific combo if you already know its name.
Add the `--no-confirm` argument to skip the confirmation dialogue.
- `comboman new` lets the user select commands to create a new combo/script/function
The `comboman new` command begins in `selection mode`, so as the upwards arrow is pressed, all the lines underneath the cursor are selected.
You can toggle in between normal and select modes by pressing space; in normal mode, you can move up or down without selecting anything, and the line underneath the cursor when the normal mode is toggled on is deselected. 
You can also deselect individual lines by pressing 'd' or left arrow.
Press Enter to enter the next screen, which will let you Edit the selection or save it as a Combo, Script or Function.
Lastly, you can use the argument `--combo-directory` to pass the path to the directory where you would like to store the combos.
