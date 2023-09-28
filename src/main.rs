use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, Url};
use web_sys::HtmlElement;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use js_sys;

#[derive(Debug, Deserialize, Serialize)]
struct Response {
    status: usize,
    status_text: String,
    http_version: String,
    headers: HashMap<String, String>,
    cookies: Vec<Cookie>,
    content: Content,
    redirect_url: String,
    headers_size: usize,
    body_size: usize,
}

#[derive(Debug, Deserialize, Serialize)]
struct Cookie {
    name: String,
    value: String,
    path: Option<String>,
    domain: Option<String>,
    expires: Option<String>,
    http_only: Option<bool>,
    secure: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Content {
    size: usize,
    mime_type: String,
    text: Option<String>,
    encoding: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Creator {
    name: String,
    version: String,
}


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
pub fn process_file(contents: &str) {
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
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let anchor = document.create_element("a").unwrap().dyn_into::<HtmlElement>().unwrap();
    anchor.set_attribute("href", &url).unwrap();
    anchor.set_attribute("download", "har.json").unwrap();
    anchor.click();

    Url::revoke_object_url(&url).unwrap();
}

fn main() {}
