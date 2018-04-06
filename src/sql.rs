//! # SQL helpers
//!
//! CockroachDB runs transactions at `SERIALIZABLE` isolation by default, which
//! implies that transactional restarts have to be taken care of. Exacerbating
//! the condition, CockroachDB is a distributed datastore and has only imperfect
//! information about conflicts, resulting in more situations that require a
//! retry. Postgres by default run at lower isolation levels and thus rarely
//! encounters these restarts, resulting in poor support for them with most
//! client drivers.
//!
//! Diesel is no exception, so we add in the missing functionality in
//! `execute_txn`.

use errors::*;

use diesel::{Connection, PgConnection};

/// Invoke the closure in the context of a retryable transaction. Errors
/// signaling a transaction retry (due to a possible serialization problem) are
/// intercepted and the operation retried transparently.
pub fn execute_txn<T, F>(conn: &PgConnection, mut op: F) -> Result<T>
where
    F: FnMut(&PgConnection) -> Result<T>,
{
    conn.transaction::<_, Error, _>(|| {
        use diesel::result::Error::DatabaseError;
        use diesel::result::DatabaseErrorKind;
        loop {
            match conn.transaction(|| op(conn)) {
                // FIXME(tschottdorf): upstream DatabaseErrorKind::SerializationFailure
                // mirroring postgres::error::T_R_SERIALIZATION_FAILURE.
                Err(Error(ErrorKind::Pg(DatabaseError(DatabaseErrorKind::__Unknown, _)), _)) => {
                    continue;
                }
                r => {
                    return r;
                }
            };
        }
    })
}

