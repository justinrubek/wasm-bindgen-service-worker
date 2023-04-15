use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    console, Navigator, RegistrationOptions, ServiceWorkerRegistration, ServiceWorkerState,
};

/// Retrieves the current service worker registration from the navigator
async fn get_service_reg(navigator: &Navigator) -> Result<ServiceWorkerRegistration, JsValue> {
    let fut = navigator.service_worker().ready()?;
    let res = JsFuture::from(fut).await?;
    Ok(ServiceWorkerRegistration::from(res))
}

fn get_worker_from_reg(reg: &ServiceWorkerRegistration) -> Option<web_sys::ServiceWorker> {
    reg.active()
        .or_else(|| reg.waiting())
        .or_else(|| reg.installing())
}

/// Creates a JS promise that resolves after the given number of milliseconds and awaits it
async fn sleep(window: &web_sys::Window, ms: i32) -> Result<(), JsValue> {
    let promise = Promise::new(&mut |resolve, _reject| {
        window
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms)
            .unwrap();
    });
    JsFuture::from(promise).await?;
    Ok(())
}

/// This is the entry point of the service worker.
/// This function is responsible for loading a service worker script from the given URL.
/// The implementation largely follows the JavaScript code above, but is written using wasm_bindgen
#[wasm_bindgen]
pub async fn register_service_worker(
    worker_url: String,
    _try_once: bool,
) -> Result<Promise, JsValue> {
    console::log_1(&"registering service worker via wasm_bindgen".into());

    let window = web_sys::window().expect("no global `window` exists");
    let location = window.location();
    let navigator = window.navigator();
    let service_worker = navigator.service_worker();

    let location_href = location.href().expect("no href found");
    let url = web_sys::Url::new_with_base(&worker_url, &location_href)?;
    let url = url.to_string().as_string().unwrap();

    let mut opts = RegistrationOptions::new();
    opts.scope("/");
    console::log_2(
        &"registering service worker with opts".into(),
        &opts.clone().into(),
    );

    let registration_fut = service_worker.register_with_options(&url, &opts);
    let registration_res = JsFuture::from(registration_fut).await?;
    let registration = ServiceWorkerRegistration::from(registration_res);

    let registered_worker = get_worker_from_reg(&registration)
        .ok_or_else(|| JsValue::from_str("Service worker registration is not valid"))?;

    console::log_2(
        &"registered service worker".into(),
        &registered_worker.clone().into(),
    );

    // Check to see if the registered worker is the same url
    if registered_worker.script_url() != url {
        console::log_1(&"registered worker is not the same url".into());

        let update_fut = registration.update()?;
        JsFuture::from(update_fut).await?;

        console::log_1(&"service worker updated".into());
    }

    // Await service worker to be ready
    let service_reg = get_service_reg(&navigator).await?;

    if navigator.service_worker().controller().is_none() {
        // TODO: Check for errors such as when calling unregister, and reload the page
        console::log_1(&"service worker is not controlling".into());

        let reg = JsFuture::from(
            navigator
                .service_worker()
                .get_registration_with_document_url("/"),
        )
        .await?;
        let reg = ServiceWorkerRegistration::from(reg);
        console::log_1(&"unregistering service worker".into());

        JsFuture::from(reg.unregister()?).await?;
        console::log_1(&"service worker unregistered, trying to re-register".into());

        location.reload()?;
        return Ok(Promise::resolve(&JsValue::NULL));
    }

    // attempt to get the service worker from the registration, if it's not there, try to re-get the registration and try again
    let service_worker = match get_worker_from_reg(&service_reg) {
        Some(worker) => worker,
        None => {
            console::log_1(&"no worker on registration, trying to re-get registration".into());
            let service_reg = get_service_reg(&navigator).await?;
            match get_worker_from_reg(&service_reg) {
                Some(worker) => worker,
                None => {
                    console::log_1(
                        &"no worker on registration, waiting a bit and trying again".into(),
                    );
                    sleep(&window, 50).await?;

                    match get_worker_from_reg(&service_reg) {
                        Some(worker) => worker,
                        None => {
                            console::log_1(&"no worker on registration, giving up".into());
                            return Err(JsValue::from_str(
                                "Service worker registration is not valid",
                            ));
                        }
                    }
                }
            }
        }
    };

    match service_worker.state() {
        ServiceWorkerState::Redundant => {
            console::log_1(&"service worker is redundant".into());
            // reload
            location.reload()?;
        }
        ServiceWorkerState::Activated => {
            console::log_1(&"service worker is activated".into());
        }
        _ => {
            console::log_1(
                &"service worker controlling, but not activated. Waiting on event".into(),
            );
            // reload the page
            location.reload()?;
        }
    }

    Ok(Promise::resolve(&JsValue::from(service_worker)))
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
fn init() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    // let service_worker_promise = register_service_worker(false);

    Ok(())
}
