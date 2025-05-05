use crate::Proxy;
use crate::futures::futures::Future;
use crate::runtime::Action;
use winit::event_loop::EventLoop;
#[cfg(target_os = "macos")]
#[allow(unused_imports)]
use winit::platform::macos::EventLoopBuilderExtMacOS;
#[cfg(target_os = "windows")]
use winit::platform::windows::{EventLoopBuilderExtWindows, WindowExtWindows};

pub fn create_event_loop<T: 'static + std::fmt::Debug + Send>()
-> (EventLoop<Action<T>>, Proxy<T>, impl Future<Output = ()>) {
    #[cfg(feature = "custom-menu")]
    {
        let mut event_loop_builder = EventLoop::<Action<T>>::with_user_event();

        // setup accelerator handler on Windows
        #[cfg(target_os = "windows")]
        {
            let menu_bar = menu_bar.clone();
            event_loop_builder.with_msg_hook(move |msg| {
                use windows_sys::Win32::UI::WindowsAndMessaging::{
                    MSG, TranslateAcceleratorW,
                };
                unsafe {
                    let msg = msg as *const MSG;
                    let translated = TranslateAcceleratorW(
                        (*msg).hwnd,
                        menu_bar.haccel() as _,
                        msg,
                    );
                    translated == 1
                }
            });
        }
        #[cfg(target_os = "macos")]
        let event_loop_builder = event_loop_builder.with_default_menu(false);

        let event_loop = event_loop_builder.build().expect("Create event loop");

        let (proxy, worker) = Proxy::<T>::new(event_loop.create_proxy());

        let p = proxy.clone();
        muda::MenuEvent::set_event_handler(Some(move |ev| {
            p.send_action(Action::Menu(ev));
        }));

        (event_loop, proxy, worker)
    }

    #[cfg(not(feature = "custom-menu"))]
    {
        let event_loop = EventLoop::<Action<T>>::with_user_event()
            .build()
            .expect("Create event loop");
        let (proxy, worker) = Proxy::<T>::new(event_loop.create_proxy());

        (event_loop, proxy, worker)
    }
}
