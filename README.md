
**whalespotter** is

* a convenient application to fast locate fat files and folders
* a demonstration of how to use channels to parallelize computations and never block the UI
* a demonstration of a few [Termimad](https://github.com/Canop/termimad/) widgets

![screen](img/screen.png)

## Installation

The simplest solution is to execute

	cargo install whalespotter

If you want to play with the code, you'll probably fetch the repository. From there you can do

	cargo build --release

and the executable will be in `target/release`.

Note that the current version of whalespotter doesn't run on windows (due to some additional code to deduplicate inodes).


## Usage

Pass the desired path:

	whalespotter ~

* Hit *ctrl-q* to quit
* *↑* and *↓* to select and *enter* to open",
* *enter* to open the selected directory (in whalespotter) or file (with `xdg-open`)
* *esc* to either unselect, or go to parent, or quit
* *pageUp* and *pageDown* to scroll
* *F5* to refresh
