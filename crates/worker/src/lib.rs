use wasm_bindgen::prelude::*;
use web_sys::{console, ServiceWorkerGlobalScope};

pub mod error;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn init_worker() -> std::result::Result<(), JsValue> {
    let global = js_sys::global();

    if let Ok(true) = js_sys::Reflect::has(&global, &JsValue::from_str("ServiceWorkerGlobalScope"))
    {
        console::log_1(&JsValue::from_str("in service worker"));
        // we're in a service worker, so we can cast the global to a ServiceWorkerGlobalScope
        let global = global.unchecked_into::<ServiceWorkerGlobalScope>();

        // Force immediate activation
        let on_install = on_install(&global)?;
        let on_activate = on_activate(&global)?;
        global.set_oninstall(Some(on_install.as_ref().unchecked_ref()));
        global.set_onactivate(Some(on_activate.as_ref().unchecked_ref()));

        // register all the other callbacks
        let on_message = on_message(&global)?;
        global.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

        // Ensure that the closures are not dropped before the service worker is terminated
        // This is technically a memory leak, but I'm not sure that it matters in this case
        on_install.forget();
        on_activate.forget();
        on_message.forget();
    } else {
        console::log_1(&JsValue::from_str("not in service worker"));
        return Err(error::Error::NotInServiceWorker.into());
    }

    Ok(())
}

fn on_install(
    global: &ServiceWorkerGlobalScope,
) -> std::result::Result<Closure<dyn FnMut(web_sys::ExtendableEvent)>, JsValue> {
    let skip_waiting = global.skip_waiting()?;
    Ok(Closure::wrap(
        Box::new(move |event: web_sys::ExtendableEvent| {
            event.wait_until(&skip_waiting).unwrap();
        }) as Box<dyn FnMut(_)>,
    ))
}

fn on_activate(
    global: &ServiceWorkerGlobalScope,
) -> std::result::Result<Closure<dyn FnMut(web_sys::ExtendableEvent)>, JsValue> {
    let clients = global.clients();
    Ok(Closure::wrap(
        Box::new(move |event: web_sys::ExtendableEvent| {
            event.wait_until(&clients.claim()).unwrap();
        }) as Box<dyn FnMut(_)>,
    ))
}

/// Displays a message in the console when a message is received from the client
fn on_message(
    _global: &ServiceWorkerGlobalScope,
) -> std::result::Result<Closure<dyn FnMut(web_sys::ExtendableMessageEvent)>, JsValue> {
    Ok(Closure::wrap(
        Box::new(move |event: web_sys::ExtendableMessageEvent| {
            console::log_2(&JsValue::from_str("sw msg:"), &event.data());
        }) as Box<dyn FnMut(_)>,
    ))
}
