# wasm-bindgen-service-worker

This is an example implementation of a [service-worker](https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API/Using_Service_Workers) written in Rust and ran using [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) and some utility functions for working with the worker.
This allows for usage of service workers in a browser context with a minimal amount of JavaScript.


The `worker` crate contains a service worker that will listen to messages and log them to the console.
The `loader` crate contains functions which can be used to register a service worker with the browser.

## usage

You can follow wasm-bindgen's [`Without a Bundler`](https://rustwasm.github.io/docs/wasm-bindgen/examples/without-a-bundler.html) example to see how to run these in the browser.
There are nix flake packages for each of these components that can help deduce the build steps. See [`flake-parts/cargo.nix`](flake-parts/cargo.nix) to see how they are built.
