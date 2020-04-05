[![MIT][s2]][l2] [![Latest Version][s1]][l1] [![Chat on Miaou][s3]][l3]

[s1]: https://img.shields.io/crates/v/whalespotter.svg
[l1]: https://crates.io/crates/whalespotter

[s2]: https://img.shields.io/badge/license-MIT-blue.svg
[l2]: LICENSE

[s3]: https://miaou.dystroy.org/static/shields/room.svg
[l3]: https://miaou.dystroy.org/3?broot

**whalespotter** is

* a convenient application to fast locate fat files and folders
* a demonstration of how to use channels to parallelize computations and never block the UI
* a demonstration of a few [Termimad](https://github.com/Canop/termimad/) widgets
* a very small and easy to read codebase

![screen](img/screen.png)

## Installation

The simplest solution is to execute

	cargo install whalespotter

If you want to play with the code, you'll probably fetch the repository. From there you can do

	cargo build --release

and the executable will be in `target/release`.

Notes:

* the current version of whalespotter doesn't run on windows (due to some additional code to deduplicate inodes). It could be easily adapted but I'd need at least a Windows tester.
* reported sizes take blocks into account, so they may be smaller than the nominal size for sparse files (the goal is to find what takes space in your disks).


## Usage

Pass the desired path:

	whalespotter ~

* Hit *ctrl-q* to quit
* *↑* and *↓* to select and *enter* to open",
* *enter* to open the selected directory (in whalespotter) or file (with `xdg-open`)
* *esc* to either unselect, or go to parent, or quit
* *pageUp* and *pageDown* to scroll
* *F5* to refresh

Whalespotter is dedicated to one use case: spotting big directories and files. If you want also other features like launching, deleting, moving, etc. you may be interested in a less focused tool, [broot](https://github.com/Canop/broot).
