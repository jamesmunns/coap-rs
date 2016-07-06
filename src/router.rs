use std::collections::HashMap;
use message::request::CoAPRequest;
use message::response::CoAPResponse;
use message::header::{MessageClass, Requests};
use message::packet::CoAPOption;
use message::IsMessage;

pub type ReqHandler = fn(CoAPRequest) -> Option<CoAPResponse>;
type HandleDispatch = HashMap<String, ReqHandler>;
pub struct CoAPRouter {
    map: HashMap<Requests, HandleDispatch>
}

impl CoAPRouter {
    pub fn new() -> CoAPRouter {
        return CoAPRouter {
            map: HashMap::new()
        }
    }
    pub fn route(&mut self, method: Requests, endpoint: String, handler: ReqHandler) {
        self.map.entry(method.clone())          // See if method already in CoAPRouter
            .or_insert(HandleDispatch::new())   //   if not, add an empty HandleDispatch
            .insert(endpoint, handler);         //   Add/Update endpoint->handler pair
    }

    pub fn get(&mut self, endpoint: String, handler: ReqHandler) {
        self.route(Requests::Get, endpoint, handler);
    }
    pub fn post(&mut self, endpoint: String, handler: ReqHandler) {
        self.route(Requests::Post, endpoint, handler);
    }
    pub fn put(&mut self, endpoint: String, handler: ReqHandler) {
        self.route(Requests::Put, endpoint, handler);
    }
    pub fn delete(&mut self, endpoint: String, handler: ReqHandler) {
        self.route(Requests::Delete, endpoint, handler);
    }

    pub fn handler(&self, req: CoAPRequest) -> Option<CoAPResponse> {
        req.get_option(CoAPOption::UriPath).and_then(|req_path| match req.get_class() {
            MessageClass::RequestType(rq_type) => {
                let path = String::from_utf8(req_path.front().unwrap().to_vec()).unwrap(); // TODO less unwrap
                self.map.get(&rq_type).and_then(|dispatch| dispatch.get(&path)).and_then(|handle| handle(req))
            }
            _ => None,
        })
    }
}
