use std::env;
use std::ops::Deref;

use dotenv;

use r2d2;
use r2d2_diesel::ConnectionManager;

use diesel::pg::PgConnection;

use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket::http::Status;


type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

/// For the convenience of using &DbConn as &diesel::pg::PgConnection.
impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Initialize a connection pool from the `DATABASE_URL` environment variable.
pub fn init_pool_from_env() -> Pool {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::new(database_url);
    let config = ::r2d2::Config::default();
    r2d2::Pool::new(config, manager).unwrap()
}
