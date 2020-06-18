use winit::event_loop::EventLoopProxy;
use objc::{
    declare::ClassDecl,
    runtime::{
        Object,
        Class,
        Sel,
    },
};

use uikit_sys::{
    id,
};


#[derive(PartialEq, Clone, Debug)]
pub struct WidgetEvent {
    widget_id: u64,
}

#[derive(Debug)]
pub struct EventHandler{
    pub id: id,
    pub widget_id: u64,
}
use std::sync::atomic::{AtomicU64, Ordering};
static mut PROXY : Option<EventLoopProxy<WidgetEvent>> = None;
static mut COUNTER: Option<AtomicU64> = None;
impl EventHandler {
    pub fn init(proxy: EventLoopProxy<WidgetEvent>) {
        unsafe {
            COUNTER = Some(AtomicU64::new(0));
            PROXY = Some(proxy);
        }
    }
    pub fn new() -> Self
    {
        let mut widget_id = 0;
        let obj = unsafe {
            let obj: id = objc::msg_send![Self::class(), alloc];
            let obj: id = objc::msg_send![obj, init];

            if let Some(counter) = &COUNTER {
                widget_id = counter.fetch_add(0, Ordering::Relaxed);
                (*obj).set_ivar::<u64>("widget_id", widget_id);
            }
            obj
        };
        Self{id: obj, widget_id}
    }
    extern "C" fn event(this: &Object, _cmd: objc::runtime::Sel)
    {
        unsafe {
            if let Some(ref proxy) = PROXY {
                let widget_id = *this.get_ivar::<u64>("widget_id");
                let _ = proxy.send_event(WidgetEvent { widget_id } );
            }
        }
    }

    fn class() -> &'static Class
    {
        let cls_name = "RustEventHandler";
        match Class::get(cls_name) {
            Some(cls) => cls,
            None => {
                let superclass = objc::class!(NSObject);
                let mut decl = ClassDecl::new(cls_name, superclass).unwrap();
                unsafe {
                    decl.add_method(
                        objc::sel!(sendEvent),
                        Self::event as extern "C" fn(&Object, Sel),
                    );
                }
                decl.add_ivar::<u64>("widget_id");
                decl.register()
            }
        }
    }
}
