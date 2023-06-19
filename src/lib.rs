use log;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(AddHeaderRoot)
    });
}}

struct AddHeaderRoot;

impl Context for AddHeaderRoot {}

impl RootContext for AddHeaderRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(AddHeader { context_id}))
    }
}

struct AddHeader {
    context_id: u32,
}

impl Context for AddHeader {}

impl HttpContext for AddHeader {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        for (name, value) in self.get_http_request_headers() {
            log::info!("#{} -> {}: {}", self.context_id, name, value);
        }

        log::info!("Writing a custom header.");
        self.set_http_request_header("From-Proxy-Wasm", Some("Hello"));
        Action::Continue
    }
}