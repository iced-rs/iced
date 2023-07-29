use std::path::PathBuf;

use notify::{EventKind, Event, RecommendedWatcher, Watcher, Config, INotifyWatcher};




pub struct Explorer {
    pub nb_deleted: i32,
    watcher: INotifyWatcher
}



#[derive(Debug, Clone)]
pub enum Message {
    Notify(Event),
    Watch(PathBuf)
}

impl Explorer {

    pub fn new(path: PathBuf) -> Self {


        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                dbg!(res).unwrap();
            },
            Config::default(),
        ).unwrap();

        watcher.watch(&path, notify::RecursiveMode::NonRecursive).unwrap();

        Explorer {
            nb_deleted: 0,
            watcher
        }

    }

    pub fn update(&mut self, message: Message) {

        match message {
            Message::Notify(event) => match event.kind {
                EventKind::Remove(_) => self.nb_deleted += 1,
                _ => {} 
            }
            Message::Watch(path) => {
                self.watcher.watch(&path, notify::RecursiveMode::NonRecursive).unwrap();
            },
        }
    }



}