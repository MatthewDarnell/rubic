use rocket::options;

const MINPASSWORDLEN: usize = 5;

/// Answers the browser's CORS preflight (OPTIONS) that precedes a JSON POST from
/// the UI; the CORS fairing attaches the Access-Control-* headers to the reply.
#[options("/<_..>")]
pub fn cors_preflight() -> &'static str {
    ""
}

pub mod info;
pub mod peer;
pub mod identity;
pub mod wallet;
pub mod transaction;
pub mod asset;
pub mod qx;