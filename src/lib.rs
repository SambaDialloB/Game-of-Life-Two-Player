use futures::{future, Future};
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response,console};
use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};


// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
#[derive(Deserialize, Serialize)]
struct SubResp {
    t: Time,
    m: Vec<MessageResp>,
}

#[derive(Deserialize, Serialize)]
struct MessageResp {
    d: Message,
}

#[derive(Deserialize, Serialize)]
struct Time {
    t: String,
}

//Message is a sub object of MessageResp
#[derive(Serialize, Deserialize)]
struct Message {
    uuid: String,
    text: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct PubResp {
    pub num: u8,
    pub sent: String,
    pub time: String,
}

#[wasm_bindgen]
pub fn publish(text: &str, channel: &str, uuid: &str) -> Promise {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let message = Message { 
        uuid: "person".to_string(),
        text: "teeext".to_string()
    };
    let m_json = serde_json::to_string(&message).unwrap();
    let url = format!(
        "https://{host}/publish/{pubkey}/{subkey}/0/{channel}/0/{message}",
        host = "ps.pndsn.com",
        pubkey = "INSERT_PUB_KEY_HERE",
        subkey = "INSERT_SUB_KEY_HERE",
        channel = percent_encode(channel.as_bytes(), PATH_SEGMENT_ENCODE_SET),
        message = percent_encode(m_json.as_bytes(), PATH_SEGMENT_ENCODE_SET),
    );
    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")
        .unwrap();

    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&request);

    let future = JsFuture::from(request_promise)
        .and_then(|resp_value| {
            // `resp_value` is a `Response` object.
            assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into().unwrap();
            resp.json()
        })
        .and_then(|json_value: Promise| {
            // Convert this other `Promise` into a rust `Future`.
            JsFuture::from(json_value)
        })
        .and_then(|json| {
            // Use serde to parse the JSON into a struct.
            let resp: PubResp = json.into_serde().unwrap();

            // Send the `Branch` struct back to JS as an `Object`.
            future::ok(JsValue::from_serde(&resp).unwrap())
        });

    // Convert this Rust `Future` back into a JS `Promise`.
    future_to_promise(future)
}

#[wasm_bindgen]
pub fn subscribe(time: &str, channel: &str) -> Promise {

    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let url = format!(
        "https://{host}/v2/subscribe/{subkey}/{channel}/0/{time}",
        host = "ps.pndsn.com",
        subkey = "INSERT_SUB_KEY_HERE",
        channel = percent_encode(channel.as_bytes(), PATH_SEGMENT_ENCODE_SET),
        time = percent_encode(time.as_bytes(), PATH_SEGMENT_ENCODE_SET),
    );

    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    request
        .headers()
        .set("Accept", "application/vnd.github.v3+json")
        .unwrap();

    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&request);
    log!("Inside subscribe loop");
    let future = JsFuture::from(request_promise)
        .and_then(|resp_value| {

            // `resp_value` is a `Response` object.
            assert!(resp_value.is_instance_of::<Response>());
            let resp: Response = resp_value.dyn_into().unwrap();
            resp.json()
        })
        .and_then(|json_value: Promise| {
            // Convert this other `Promise` into a rust `Future`.
            JsFuture::from(json_value)
        })
        .and_then(|json| {
            // Use serde to parse the JSON into a struct.
            let resp: SubResp = json.into_serde().unwrap();
            // Send the `Branch` struct back to JS as an `Object`.
            future::ok(JsValue::from_serde(&resp).unwrap())
        });

    // Convert this Rust `Future` back into a JS `Promise`.
    future_to_promise(future)
    
}