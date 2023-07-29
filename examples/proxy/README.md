this example show how having Proxy could be useful.


the important code part in here:

```
let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                proxy.send(AppMsg::Event(res.unwrap()))
            },
            Config::default(),
        )
        .unwrap();

```

notify::RecommendedWatcher have his own runtime to watch event on fs.

Currently, the only way to use it in Iced is by using subscription. This is not easy to use and can even lead to issues.

imagine you want to see what files are in a dir, and also be notified when fs events occurs in it.

# with Subscription
```
sender.send(Action::Watch(path)) // sender is send by using subscription::channel

// ... time ... (another process)
// anything on the fs could happend (lead to error)

fetch_dir(path)
```

# with proxy
```
// we can have the watcher inside our struct
watcher.watch(path)
fetch_dir(path)
```