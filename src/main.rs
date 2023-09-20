use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, File, FileReader, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use js_sys;

#[derive(Deserialize, Serialize)]
struct Har {
    log: Log,
}

impl Default for Har {
  fn default() -> Self {
      Har {
          log: Log {
              version: String::new(),
              pages: None,
              entries: Vec::new(),
              creator: HashMap::new(),
          },
      }
  }
}

#[derive(Deserialize, Serialize)]
struct Log {
    version: String,
    creator: HashMap<String, Value>,
    pages: Option<Vec<HashMap<String, Value>>>,
    entries: Vec<Entry>,
}

#[derive(Deserialize, Serialize)]
struct Entry {
    #[serde(flatten)]
    other: HashMap<String, Value>,
    request: Request,
}

#[derive(Deserialize, Serialize)]
struct Request {
    #[serde(flatten)]
    other: HashMap<String, Value>,
    headers: Vec<Header>,
}

#[derive(Deserialize, Serialize)]
struct Header {
    name: String,
    value: String,
}

fn strip_auth_headers(har: &mut Har) {
    for entry in &mut har.log.entries {
        entry.request.headers.retain(|header| header.name != "authorization");
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn process_file(file: File) {
    let reader = FileReader::new().unwrap();
    let onload = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let reader = event.target().unwrap().dyn_into::<FileReader>().unwrap();
        let contents = reader.result().unwrap().as_string().unwrap();

        // Parse the HAR file
        let mut har: Har = serde_json::from_str(&contents).unwrap_or_else(|err| {
            log(&format!("Error parsing HAR file: {:?}", err));
            Har::default()
        });

        // Strip out the Authorization headers
        strip_auth_headers(&mut har);



        // Serialize back to JSON
        let output = serde_json::to_string_pretty(&har).unwrap();
        let output_js_value = JsValue::from_str(&output);
        let blob = Blob::new_with_str_sequence(&js_sys::Array::of1(&output_js_value)).unwrap();
        let url = Url::create_object_url_with_blob(&blob).unwrap();

        // Trigger download
        web_sys::window().unwrap().open_with_url_and_target(&url, "_blank").unwrap();

        Url::revoke_object_url(&url).unwrap();
    }) as Box<dyn FnMut(_)>);

    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    reader.read_as_text(&file).unwrap();
    onload.forget();
}

fn main() {}
