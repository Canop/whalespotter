use crate::file_info::FileInfo;
use crossbeam::channel::{unbounded, Receiver, Sender};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

pub enum ComputationEvent {
    Finished,
    FileInfo(FileInfo), // TODO add progress
}

pub struct Computer {
    tx: Sender<ComputationEvent>,
    pub rx: Receiver<ComputationEvent>,
    task_count: Arc<AtomicUsize>,
}

impl Computer {
    pub fn new() -> Self {
        let (tx, rx) = unbounded();
        let task_count = Arc::new(AtomicUsize::new(0));
        Self { tx, rx, task_count }
    }
    pub fn do_children(&mut self, root: &Path) {
        lazy_static! {
            static ref PROC: PathBuf = Path::new("/proc").to_path_buf();
        }
        let thread_count = Arc::new(AtomicUsize::new(0));
        let start_task_count = self.task_count.fetch_add(1, Ordering::Relaxed) + 1;
        for entry in root.read_dir().expect("read_dir call failed").flatten() {
            if entry.path() == *PROC {
                continue; // size of this dir doesn't mean anything useful, let's just forget it
            }
            if let Ok(md) = entry.metadata() {
                if md.is_file() {
                    self.tx
                        .send(ComputationEvent::FileInfo(FileInfo {
                            path: entry.path(),
                            file_count: 1,
                            size: md.len(),
                            is_dir: false,
                        }))
                        .unwrap();
                } else if md.is_dir() {
                    let tx = self.tx.clone();
                    thread_count.fetch_add(1, Ordering::Relaxed);
                    let thread_count = Arc::clone(&thread_count);
                    let task_count = Arc::clone(&self.task_count);
                    thread::spawn(move || {
                        let fi = FileInfo::from_dir(entry.path());
                        // we check we didn't finish an obsolete task
                        let current_task_count = task_count.load(Ordering::Relaxed);
                        if current_task_count != start_task_count {
                            return;
                        }
                        tx.send(ComputationEvent::FileInfo(fi)).unwrap();
                        let remaining_thread_count = thread_count.fetch_sub(1, Ordering::Relaxed);
                        if remaining_thread_count == 1 {
                            tx.send(ComputationEvent::Finished).unwrap();
                        }
                    });
                }
            }
        }
        if thread_count.load(Ordering::Relaxed) == 0 {
            // there was no folder
            self.tx.send(ComputationEvent::Finished).unwrap();
        }
    }
}
