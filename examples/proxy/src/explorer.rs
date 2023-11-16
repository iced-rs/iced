use notify::{Config, Event, EventKind, INotifyWatcher, RecommendedWatcher, Watcher};

pub struct Explorer {
    pub nb_deleted: i32,
    #[allow(dead_code)]
    watcher: INotifyWatcher,
}

impl Explorer {
    pub fn new(path: PathBuf, proxy: Proxy<AppMsg>) -> Option<Self> {
        if !path.is_dir() {
            println!("path is not a dir");
            return None;
        }

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                proxy.send(AppMsg::Event(res.unwrap()))
            },
            Config::default(),
        )
        .unwrap();

        watcher
            .watch(&path, notify::RecursiveMode::NonRecursive)
            .unwrap();

        println!("watching {}", path.to_string_lossy());

        let explorer = Explorer {
            nb_deleted: 0,
            watcher,
        };

        Some(explorer)
    }

    pub fn process_event(&mut self, event: Event) {
        if let EventKind::Remove(_) = event.kind {
            self.nb_deleted += 1;
        }
    }
}
