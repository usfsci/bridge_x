use serde_json::Value;
use bytes::{Bytes};
use std::{fmt};
use thiserror::Error;


const PROTO_VERSION: &str  = "SMSG/0.1";
const START_LINE_ITEMS: usize = 4;

#[derive(Error, Debug)]
pub enum MessageError {
    #[error("custom: {0}")]
    Custom(String),

    #[error("no start line: {0}")]
    NoStartLine(String),

    #[error("malformed start line: {0}")]
    MalformedStartLine(String),

    #[error("received bytes are not UTF8: {0}")]
    ParseUtf8(#[from] std::str::Utf8Error),

    #[error("json decode error: {0}")]
    DecodeJson(#[from] serde_json::Error),
}

enum RequestTokenIndex {
    IdIndex = 0,
    ActionIndex,
    KindIndex,
    ProtoIndex
}

enum ResponseTokenIndex {
    ProtoIndex = 0,
    IdIndex,
    CodeIndex,
    TextIndex
}

pub struct Request {
    pub protocol: String,
    pub version: String,
    pub id: usize,
    pub action: String,
    pub kind: String,
    pub body: Option<Value>,
}

impl Request {
    pub fn new(protocol: String, version: String, id: usize, action: String, kind: String, body: Option<Value>) -> Self {
        Request {
            protocol,
            version,
            id,
            action,
            kind,
            body,
        }
    }

    pub fn encode(&self) -> String {
        let sl = format!("{}/{} {} {} {}\n", self.protocol, self.version, self.id, self.action, self.kind);

        // Don't move out of self.body; borrow and format the JSON payload if present.
        let body = match &self.body {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };

        let s = format!("{}{}", sl, body);

        s
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json = self.body.as_ref().unwrap_or_default();

        write!(f, "proto: {}\nver: {}\nid: {}\naction: {}\nkind: {}\nbody: {}\n", 
            self.protocol, self.version, self.id, self.action, self.kind, json)
    }
}

pub struct Response {
    pub protocol: String,
    pub version: String,
    pub id: usize,
    pub code: usize,
    pub text: String,
    pub body: Option<Value>
}

impl Response {
    pub fn new(protocol: String, version: String, id: usize, code: usize, text: String, body: Option<Value>) -> Self {
        Response {
            protocol,
            version,
            id,
            code,
            text,
            body,
        }
    }

    pub fn encode(&self) -> String {
        let sl = format!("{}/{} {} {} {}\n", self.protocol, self.version, self.id, self.code, self.text);

        // Don't move out of self.body; borrow and format the JSON payload if present.
        let body = match &self.body {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };

        let s = format!("{}{}", sl, body);

        s
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let json = self.body.as_ref().unwrap_or_default();

        write!(f, "proto: {}\nver: {}\nid: {}\ncode: {}\ntext: {}\nbody: {}\n", 
            self.protocol, self.version, self.id, self.code, self.text, json)
    }
}

pub enum Message {
    Request(Request),
    Response(Response)
}


pub fn decode_message(payload: Bytes) -> Result<Message, MessageError> {
    // All messages are ASCII and have an start_line followed by '\n'.
    // They may have or not a body after the '\n'.
    // If present the body will be JSON.

    let mut iter = payload.split(|c| {
        *c == '\n' as u8
    });

    // Get the start line or return error
    let l = iter.next().ok_or(MessageError::NoStartLine("".to_string()))?;

    //let line: &str;
    let line = match std::str::from_utf8(l) {
        Ok(s) => s,
        Err(e) => {
            return Err(MessageError::ParseUtf8(e));
        }
    };

    // get the body as an option
    let json: Option<Value> = match iter.next() {
        Some(a) => {
            match serde_json::from_slice::<Value>(&a) {
                Ok(j) => Some(j),
                Err(e) => {
                    eprintln!("JSON parse error: {} -- raw={:?}", e, payload);
                    return Err(MessageError::DecodeJson(e));
                    // optionally send an error response
                }
            }
        },
        None => None
    };

    build_message(line, json)
}

fn build_message(line: &str, json: Option<Value>) -> Result<Message, MessageError> {
    //let sline = line.to_ascii_lowercase();
    let tokens: Vec<&str> = line.split_ascii_whitespace().collect();

    if tokens.len() != START_LINE_ITEMS {
        return Err(MessageError::MalformedStartLine(std::format!("expected {} tokens, got {}", START_LINE_ITEMS, tokens.len())));
    } 

    if tokens[0] == PROTO_VERSION {
        // This is a response
        let response = build_response(&tokens, json)?;
        Ok(Message::Response(response))
    } else {
        // This is a request
        let request = build_request(&tokens, json)?;
        Ok(Message::Request(request))
    }
}

fn build_request(tokens: &Vec<&str>, body: Option<Value>) -> Result<Request, MessageError> {
    // Parse id
    let id = match tokens[RequestTokenIndex::IdIndex as usize].parse::<usize>() {
        Ok(v) => v,
        Err(_) => return Err(MessageError::Custom("invalid id".to_string())),
    };

    let action = tokens[RequestTokenIndex::ActionIndex as usize].to_string();
    let kind = tokens[RequestTokenIndex::KindIndex as usize].to_string();

    // tokens[3] expected to be protocol/version like "SMSG/0.1"
    let proto_ver = tokens[RequestTokenIndex::ProtoIndex as usize];
    let mut parts = proto_ver.splitn(2, '/');
    let protocol = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();

    Ok(Request { protocol, version, id, action, kind, body })
}

fn build_response(tokens: &Vec<&str>, body: Option<Value>) -> Result<Response, MessageError> {
    // Parse id
    let id = match tokens[ResponseTokenIndex::IdIndex as usize].parse::<usize>() {
        Ok(v) => v,
        Err(_) => return Err(MessageError::Custom("id is non num".to_string())),
    };

    let code = match tokens[ResponseTokenIndex::CodeIndex as usize].parse::<usize>() {
        Ok(v) => v,
        Err(_) => return Err(MessageError::Custom("code is non num".to_string())),
    };

    let text = tokens[ResponseTokenIndex::TextIndex as usize].to_string();

    // tokens[0] expected to be protocol/version like "SMSG/0.1"
    let proto_ver = tokens[ResponseTokenIndex::ProtoIndex as usize];
    let mut parts = proto_ver.splitn(2, '/');
    let protocol = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();

    Ok(Response { protocol, version, id, code, text, body })
}   
