use crate::futures::futures::channel::mpsc;
use crate::runtime::Task;

use std::thread;

pub fn spawn_blocking<T>(
    f: impl FnOnce(mpsc::Sender<T>) + Send + 'static,
) -> Task<T>
where
    T: Send + 'static,
{
    let (sender, receiver) = mpsc::channel(1);

    let _ = thread::spawn(move || {
        f(sender);
    });

    Task::stream(receiver)
}
