use wasm_bindgen::prelude::*;
extern crate stdweb;
use stdweb::web::{WebSocket, event::SocketMessageEvent, event::IMessageEvent, IEventTarget};
use plotters::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;
//chrono is required since std::time doesn't currently work in wasm
extern crate chrono;
use chrono::prelude::*;
extern crate console_error_panic_hook;
use core::result::Result;
use std::panic;

#[macro_use]
extern crate serde_derive;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}


#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// Type alias for the result of a drawing function.
pub type DrawResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Type used on the JS side to convert screen coordinates to chart
/// coordinates.
#[wasm_bindgen]
pub struct Chart {
}

//JSON object being sent from Vert.x
#[derive(Serialize, Deserialize, Debug)]
pub struct Values {
    pub time: f32,
    pub value: f32,
}

//Get the window for rendering
fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

//Schedule a closure to be executed with a new animation frame
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen]
impl Chart {

    //Connect with a websocket to the given url to receive the values to be rendered
    pub fn websocket_values(ws_url: &str) -> Result<(), JsValue> {
        console_log!("Connecting to {} and rendering results in plot", ws_url);
        //Add hook for reporting failures to the console ... an real life saver
        panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mailbox = Mailbox::new(ws_url);
        let websocket_values = mailbox.received_messages.clone();
        let mut received_values:Vec<(f32, f32)> = Vec::new();
        let reference_counter_for_closure = Rc::new(RefCell::new(None));
        let reference_for_starting_render_loop = reference_counter_for_closure.clone();
        let mut time_of_last_animation_frame = Utc::now().timestamp_millis();
        let mut index_in_values = 0;
        let width = 50;

        //Prepare closure for looping the animation
        *reference_for_starting_render_loop.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let mut now = Utc::now().timestamp_millis();

            websocket_values.borrow_mut().iter().for_each(|v| {
                //deserialize values sent from Vert.x
                let v: Values = serde_json::from_str(v.as_str()).unwrap();

                //push values into the storage vector
                received_values.push((v.time, v.value ));
            });

            //clear stored values after processing
            websocket_values.borrow_mut().clear();
            if now - time_of_last_animation_frame > 100 {
                time_of_last_animation_frame = now;

                //start moving the when we have more values than the plot is wide
                if (received_values.len() - index_in_values) > width {
                    Chart::draw_range(received_values[index_in_values..index_in_values + width].to_vec(), width);
                    index_in_values += 1;
                } else if received_values.len() > 2 {
                    Chart::draw_range(received_values.to_vec(), width);
                }
            }

            //reschedule drawing
            request_animation_frame(reference_counter_for_closure.borrow().as_ref().unwrap());
        }) as Box<dyn FnMut()>));

        //Start the animation loop
        request_animation_frame(reference_for_starting_render_loop.borrow().as_ref().unwrap());
        Ok(())
    }

    fn draw_range(data: Vec<(f32,f32)>, width: usize) {
        let backend = CanvasBackend::new("canvas").expect("cannot find canvas");
        let root = backend.into_drawing_area();
        let font: FontDesc = ("sans-serif", 20.0).into();
        root.fill(&WHITE).unwrap();

        let start = data.first().unwrap().0;

        //build the cart for rendering
        let mut chart = ChartBuilder::on(&root)
            .caption("Woohoo", font)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_ranged(start .. (start + width as f32), -1f32..1f32).unwrap();

        chart.configure_mesh().x_labels(10).y_labels(10).draw().unwrap();

        chart.draw_series(LineSeries::new(
            data,
            &RED,
        )).unwrap();

        //reveal the drawn plot
        root.present().unwrap();
    }
}

pub struct Mailbox {
    socket : WebSocket,
    received_messages : Rc<RefCell<Vec<String>>>,
}


impl Mailbox {
    fn new (url: &str) -> Self {
        // create a new socket and the vector of received messages
        let socket = WebSocket::new(url).expect("No Socket :(");

        let received_messages : Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(vec![]));

        // clone the Rc, move it into a closure, use closure as event listener
        let closure_messages = received_messages.clone();

        let closure = move |mes : SocketMessageEvent| -> () {
            let rec = mes.data().into_text().unwrap();
            closure_messages.borrow_mut().push(rec);
        };
        socket.add_event_listener(closure);

        return Mailbox { socket, received_messages }
    }
}