use rocket::options;

const MINPASSWORDLEN: usize = 5;

/// Answers the browser's CORS preflight (OPTIONS) that precedes a JSON POST from
/// the UI; the CORS fairing attaches the Access-Control-* headers to the reply.
#[options("/<_..>")]
pub fn cors_preflight() -> &'static str {
    ""
}

/// Origins allowed to call the local API from a browser context: the Tauri
/// window and the Vite dev server. Any other page (a random site open in a
/// browser on this machine) gets no Access-Control-Allow-Origin header and is
/// blocked by the browser. Non-browser clients are unaffected: CORS is a
/// browser-side rule, not authentication.
pub const ALLOWED_ORIGINS: [&str; 6] = [
    "tauri://localhost",
    "http://tauri.localhost",
    "https://tauri.localhost",
    "http://localhost:5173",
    "http://127.0.0.1:5173",
    "http://localhost:1420",
];

/// The origin to echo back in Access-Control-Allow-Origin, if the request's
/// Origin is one of ours.
pub fn cors_allowed_origin(origin: Option<&str>) -> Option<&'static str> {
    let origin = origin?;
    ALLOWED_ORIGINS.iter().copied().find(|allowed| allowed.eq_ignore_ascii_case(origin))
}

pub mod info;
pub mod peer;
pub mod identity;
pub mod wallet;
pub mod transaction;
pub mod asset;
pub mod qx;