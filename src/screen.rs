use crossterm::{
    queue,
    style::{Attribute, Color::*},
    terminal::{self, Clear, ClearType},
};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use termimad::{
    ansi, Alignment, Area, CompoundStyle, ListView, ListViewCell, ListViewColumn, MadSkin,
    ProgressBar, Result,
};

use crate::file_info::FileInfo;

pub struct Screen<'t> {
    root: PathBuf,
    finished: bool,
    pub list_view: ListView<'t, FileInfo>,
    skin: &'t MadSkin,
    dimensions: (u16, u16),
    total_size: Arc<AtomicU64>,
}
impl<'t> Screen<'t> {
    pub fn new(root: PathBuf) -> Self {
        lazy_static! {
            static ref SKIN: MadSkin = make_skin();
        }
        let total_size = Arc::new(AtomicU64::new(0));
        let column_total_size = Arc::clone(&total_size);
        let columns = vec![
            ListViewColumn::new(
                "name",
                10,
                50,
                Box::new(|fi: &FileInfo| {
                    ListViewCell::new(
                        fi.path.file_name().unwrap().to_string_lossy().to_string(),
                        if fi.is_dir {
                            &SKIN.bold
                        } else {
                            &SKIN.paragraph.compound_style
                        },
                    )
                }),
            )
            .with_align(Alignment::Left),
            ListViewColumn::new(
                "items",
                7,
                9,
                Box::new(|fi: &FileInfo| {
                    ListViewCell::new(u64_to_str(fi.file_count), &SKIN.paragraph.compound_style)
                }),
            )
            .with_align(Alignment::Right),
            ListViewColumn::new(
                "size",
                5,
                6,
                Box::new(|fi: &FileInfo| {
                    ListViewCell::new(u64_to_str(fi.size), &SKIN.paragraph.compound_style)
                }),
            )
            .with_align(Alignment::Right),
            ListViewColumn::new(
                "size",
                13,
                13,
                Box::new(move |fi: &FileInfo| {
                    let total_size = column_total_size.load(Ordering::Relaxed);
                    ListViewCell::new(
                        if total_size > 0 {
                            let part = (fi.size as f32) / (total_size as f32);
                            format!("{:>3.0}% {}", 100.0 * part, ProgressBar::new(part, 8))
                        } else {
                            "".to_owned()
                        },
                        if fi.is_dir {
                            &SKIN.bold
                        } else {
                            &SKIN.paragraph.compound_style
                        },
                    )
                }),
            )
            .with_align(Alignment::Left),
        ];
        let area = Area::new(0, 1, 10, 10);
        let mut list_view = ListView::new(area, columns, &SKIN);
        list_view.sort(Box::new(|a, b| b.size.cmp(&a.size)));
        Self {
            root,
            skin: &SKIN,
            list_view,
            dimensions: (0, 0),
            total_size,
            finished: false,
        }
    }
    pub fn set_new_root(&mut self, path: PathBuf) {
        self.root = path;
        self.total_size.store(0, Ordering::Relaxed);
        self.list_view.clear_rows();
        self.finished = false;
    }
    pub fn set_finished(&mut self) {
        self.finished = true;
    }
    pub fn add_to_total_size(&mut self, to_add: u64) {
        self.total_size.fetch_add(to_add, Ordering::Relaxed);
    }
    pub fn get_root(&self) -> &Path {
        &self.root
    }
    pub fn display<W>(&mut self, writer: &mut W) -> Result<()>
    where
        W: std::io::Write,
    {
        let (width, height) = terminal::size()?;
        if (width, height) != self.dimensions {
            queue!(writer, Clear(ClearType::All))?;
            self.dimensions = (width, height);
            self.list_view.area.width = width;
            self.list_view.area.height = height - 4;
            self.list_view.update_dimensions();
        }
        let title = if self.finished {
            format!("# **{}**", self.root.as_os_str().to_string_lossy())
        } else {
            format!(
                "# **{}** *computing...*",
                self.root.as_os_str().to_string_lossy()
            )
        };
        self.skin
            .write_in_area_on(writer, &title, &Area::new(0, 0, width, 1))?;
        self.skin.write_in_area_on(
            writer,
            "Hit *ctrl-q* to quit, *esc* to go to parent, *↑* and *↓* to select, and *enter* to open",
            &Area::new(0, height-2, width, 1),
        )?;
        self.list_view.write_on(writer)
    }
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.headers[0].compound_style = CompoundStyle::with_attr(Attribute::Bold);
    skin.headers[0].align = Alignment::Left;
    skin.italic.set_fg(ansi(225));
    skin.bold = CompoundStyle::with_fg(Blue);
    skin
}

const SIZE_NAMES: &[&str] = &["", "K", "M", "G", "T", "P", "E", "Z", "Y"];
/// format a number of as a string
pub fn u64_to_str(mut v: u64) -> String {
    let mut i = 0;
    while v >= 2300 && i < SIZE_NAMES.len() - 1 {
        v >>= 10;
        i += 1;
    }
    format!("{}{}", v, &SIZE_NAMES[i])
}
