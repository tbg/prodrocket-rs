//! # CockroachDB/Diesel/Rocket Example App
//!
//! This is an example Rocket app backed by CockroachDB, using Diesel to build
//! queries. The example uses a connection pool, handles errors properly, and
//! generally tries to follow best practices while getting you off the ground
//! quickly. It will work with Postgres instead of CockroachDB, too!
//!
//! Check out the building blocks below:
//!
//! * [CockroachDB Homepage](https://cockroachlabs.com/)
//! * [Diesel](https://diesel.rs)
//! * [Rocket](https://rocket.rs)
#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]
#![feature(custom_derive)]

extern crate dotenv;
extern crate rocket_contrib;
extern crate serde;


#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate r2d2;
extern crate r2d2_diesel;
extern crate r2d2_postgres;
extern crate rocket;
extern crate uuid;

/// The database schema.
pub mod schema;
/// Models for this app.
pub mod models;
/// Connection pooling.
pub mod connpool;
/// SQL helpers for interacting with CockroachDB.
pub mod sql;
/// Wrapper around uuid::Uuid for use with Rocket.
pub mod uuid_wrapper;

/// The error chain for this app.
pub mod errors {
    error_chain! {
        foreign_links {
            PoolTimeout(super::r2d2::GetTimeout);
            Pg(super::diesel::result::Error);
        }
    }
}

use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::http::Status;
use rocket_contrib::Json;
use uuid_wrapper::Uuid;

use diesel::prelude::*;
use models::{NewPost, Post};

use errors::*;


/// This app prevents leaking errors to the client, returning a blank internal
/// server error instead. The actual error is logged to stdout.
///
/// Note that at the time of writing, Rocket doesn't quite have its logging
/// story figured out (or something more idiomatic would happen here).
impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> ::std::result::Result<Response<'r>, Status> {
        println!("error handling request: {}", self);
        Err(Status::InternalServerError)
    }
}

/// A simple string-based response. This app typically returns it as JSON.
#[derive(Serialize)]
struct MessageResponse {
    msg: String,
}

impl MessageResponse {
    /// Create an "ok" string.
    fn ok() -> Self {
        MessageResponse {
            msg: String::from("ok"),
        }
    }
}

#[derive(FromForm, Serialize)]
struct FooRequest {
    uuid: Uuid,
    version: String,
    platform: String,
    force_unstable: bool,
    insecure: bool,
}

#[get("/foo?<r>")]
fn foo(conn: connpool::DbConn, r: FooRequest) -> Result<Json<MessageResponse>> {
   sql::execute_txn(&*conn, |c: &diesel::PgConnection| -> Result<Post> {
        let new_post = NewPost {
            title: "test title",
            body: "test body",
        };
        ::diesel::insert(&new_post)
            .into(::schema::posts::table)
            .get_result(c)
            .map_err(|e| e.into())
    })?;
    Ok(Json(MessageResponse::ok()))
}

#[error(500)]
fn handle_500(_: &Request) -> Json<MessageResponse> {
    Json(MessageResponse {
        msg: String::from("internal server error"),
    })
}

#[error(404)]
fn handle_404(_: &Request) -> Json<MessageResponse> {
    Json(MessageResponse {
        msg: String::from("not found"),
    })
}

fn main() {
    rocket::ignite()
        .mount("/api/v1", routes![foo])
        .manage(connpool::init_pool_from_env())
        .catch(errors![handle_500, handle_404])
        .launch();
}
