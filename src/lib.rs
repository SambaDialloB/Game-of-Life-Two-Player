use futures::{future, Future};
use js_sys::{Promise, Reflect};
use percent_encoding::{percent_encode, PATH_SEGMENT_ENCODE_SET};
use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use wasm_bindgen_futures::JsFuture;
use web_sys::{console, Request, RequestInit, RequestMode, Response};

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}
#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}
impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
    fn set_cell(&mut self, alive: bool) {
        *self = match alive {
            true => Cell::Alive,
            _ => Cell::Dead,
        };
    }
}
#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}
#[derive(Deserialize, Serialize, Debug)]
struct SubResp {
    t: Time,
    m: Vec<MessageResp>,
}

#[derive(Deserialize, Serialize, Debug)]
struct MessageResp {
    d: RCCMessage,
}

#[derive(Deserialize, Serialize, Debug)]
struct Time {
    t: String,
}

//Message is a sub object of MessageResp
#[derive(Deserialize, Serialize, Debug)]
struct RCCMessage {
    tick: bool,
    cells: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct PubResp {
    pub num: u8,
    pub sent: String,
    pub time: String,
}
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
#[wasm_bindgen]
impl Universe {
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }
    pub fn set_cell(&mut self, row: u32, column: u32, alive: bool) {
        let idx = self.get_index(row, column);
        self.cells[idx].set_cell(alive);
    }
    pub fn new(size: u32) -> Universe {
        let cells = (0..size * size)
            .map(|i| {
                if i % 2 == 0 || i % 7 == 0 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width: size,
            height: size,
            cells,
        }
    }
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }
    pub fn render(&self) -> String {
        self.to_string()
    }
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }
        self.cells = next;
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

#[wasm_bindgen]
pub fn publish(pub_object: &JsValue, channel: &str) -> Promise {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let messages: RCCMessage = pub_object.into_serde().unwrap();
    let m_json = serde_json::to_string(&messages).unwrap();
    let message_encoded = percent_encode(m_json.as_bytes(), PATH_SEGMENT_ENCODE_SET);
    log!("{}", message_encoded.to_string());
    let url = format!(
        "https://{host}/publish/{pubkey}/{subkey}/0/{channel}/0/{message}",
        host = "ps.pndsn.com",
        pubkey = "INSET_PUB_KEY_HERE",
        subkey = "INSET_SUB_KEY_HERE",
        channel = percent_encode(channel.as_bytes(), PATH_SEGMENT_ENCODE_SET),
        message = message_encoded,
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
        subkey = "INSET_SUB_KEY_HERE",
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
            future::ok(JsValue::from_serde(&resp).unwrap())
        });

    // // Convert this Rust `Future` back into a JS `Promise`.
    future_to_promise(future)
}
