use std::future::Future;

pub fn thread_context() -> glib::MainContext {
    glib::MainContext::get_thread_default().unwrap_or_else(|| {
        let ctx = glib::MainContext::new();
        ctx.push_thread_default();
        ctx
    })
}

pub fn spawn<F: Future<Output = ()> + 'static>(future: F) {
    glib::MainContext::default().spawn_local(future);
}


pub fn block_on<T, F: Future<Output = T> + 'static>(future: F) {
    glib::MainContext::default().block_on(future);
}