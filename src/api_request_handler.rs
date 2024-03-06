use std::collections::HashMap;

pub mod info;

#[derive(Debug, Clone)]
pub struct request {
    path: String,
    ops: Option<HashMap<String, String>>,
    function: Option<fn() -> String>
}

impl request {
    pub fn new(path: &str, ops: Option<HashMap<String, String>>, func: Option<fn() -> String>) -> Self {
        request {
            path: String::from(path),
            ops,
            function: func
        }
    }
}


pub fn submit_task(path: &str, ops: Option<&HashMap<String, String>>) {
    let available_requests: Vec<request> = info::get_routes();
    let mut found_index: i32 = -1;
    for (index, req) in available_requests.iter().enumerate() {
        if req.path == path {
            found_index = index as i32;
            break;
        }
    }
    if found_index < 0 {
        println!("Received Invalid Api! {}", path);
        return;
    }
    println!("Found Api Task To Execute {:?}", available_requests[found_index as usize]);
    if available_requests[found_index as usize].function.is_some() {
        println!("Executing Callback");
        let func = available_requests[found_index as usize].function.unwrap();
        let response = func();
        println!("CallBack Responded With: {}", &response);
    }
}