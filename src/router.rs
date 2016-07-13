use std::collections::HashMap;
use message::request::CoAPRequest;
use message::response::CoAPResponse;
use message::header::{MessageClass, Requests};
use message::packet::CoAPOption;
use message::IsMessage;

pub type ReqHandler = fn(CoAPRequest) -> Option<CoAPResponse>;
type HandleDispatch = HashMap<String, ReqHandler>;

#[derive(Clone)]
pub struct CoAPRouter {
    map: HashMap<Requests, HandleDispatch>,
}

impl CoAPRouter {
    pub fn new() -> CoAPRouter {
        return CoAPRouter { map: HashMap::new() };
    }
    pub fn route(&mut self, method: Requests, endpoint: &String, handler: ReqHandler) {
        self.map.entry(method.clone())              // See if method already in CoAPRouter
            .or_insert(HandleDispatch::new())       //   if not, add an empty HandleDispatch
            .insert(endpoint.clone(), handler);     //   Add/Update endpoint->handler pair
    }

    pub fn get(&mut self, endpoint: &String, handler: ReqHandler) {
        self.route(Requests::Get, endpoint, handler);
    }
    pub fn post(&mut self, endpoint: &String, handler: ReqHandler) {
        self.route(Requests::Post, endpoint, handler);
    }
    pub fn put(&mut self, endpoint: &String, handler: ReqHandler) {
        self.route(Requests::Put, endpoint, handler);
    }
    pub fn delete(&mut self, endpoint: &String, handler: ReqHandler) {
        self.route(Requests::Delete, endpoint, handler);
    }

    pub fn handle(&self, req: CoAPRequest) -> Option<CoAPResponse> {
        // Obtain first URI, if there is one
        //   NOTE: only the first URI is handled. Others ignored
        req.get_option(CoAPOption::UriPath)
            .and_then(|uri_ll| {
                uri_ll.front().and_then(|first_uri| String::from_utf8(first_uri.to_vec()).ok())
            })
            .and_then(|path| {
                // Verify this is a request classed packet
                match req.get_class() {
                    MessageClass::RequestType(rq_type) => {
                        self.map
                            .get(&rq_type)
                            .and_then(|dispatch| dispatch.get(&path))
                            .and_then(|handle| handle(req))
                    }
                    _ => None,
                }
            })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use message::request::CoAPRequest;
    use message::response::CoAPResponse;
    use message::packet::CoAPOption;
    use message::header::{MessageClass, Requests, MessageType};
    use message::IsMessage;

    fn echo_handler(request: CoAPRequest) -> Option<CoAPResponse> {
        let uri_path = request.get_option(CoAPOption::UriPath).unwrap();
        let mut response = request.response.unwrap();
        response.set_payload(uri_path.front().unwrap().clone());

        Some(response)
    }

    #[test]
    fn basic_test() {
        let mut req_1 = CoAPRequest::new();
        req_1.add_option(CoAPOption::UriPath, b"foo".to_vec());
        req_1.set_class(MessageClass::RequestType(Requests::Get));
        req_1.set_type(MessageType::Confirmable);
        req_1.response = CoAPResponse::new(&req_1.message);

        let mut req_2 = CoAPRequest::new();
        req_2.add_option(CoAPOption::UriPath, b"bar".to_vec());
        req_2.set_class(MessageClass::RequestType(Requests::Get));
        req_2.set_type(MessageType::Confirmable);
        req_2.response = CoAPResponse::new(&req_1.message);

        let mut req_3 = CoAPRequest::new();
        req_3.add_option(CoAPOption::UriPath, b"foo".to_vec());
        req_3.set_class(MessageClass::RequestType(Requests::Post));
        req_3.set_type(MessageType::Confirmable);
        req_3.response = CoAPResponse::new(&req_3.message);

        let req_4 = req_1.clone();

        let mut rtr = CoAPRouter::new();
        rtr.get(&"foo".to_string(), echo_handler);

        assert!(rtr.handle(req_1).is_some());
        assert!(rtr.handle(req_2).is_none());
        assert!(rtr.handle(req_3).is_none());

        assert_eq!(b"foo".to_vec(), rtr.handle(req_4).unwrap().message.payload);
    }
}
