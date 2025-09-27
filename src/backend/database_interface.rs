use std::str::FromStr;

use iroh::SecretKey;

use super::database::{DataLink, DatabaseParam, DatabaseParams};
use super::sql::*;

use rand::rngs::OsRng;

pub struct DatabaseInterface;

impl DatabaseInterface {

    pub fn make_tables_nonblocking(db: DataLink) {
        db.execute(CREATE_NODEID_TABLE, DatabaseParams::empty());
    }

    pub fn get_node_id_blocking(db: DataLink) -> SecretKey {

        let mut rng = OsRng;
        
        if let Ok(results) = db.query_blocking(SELECT_NODEID, DatabaseParams::empty()) {
            if let Some(row) = results.get(0) {
                if let Some(node_id) = row.get(0) {
                    if let Ok(secret_key) = hex::decode(node_id.string()) {
                        if let Ok(bytecode) = &secret_key.try_into() {
                            return SecretKey::from_bytes(bytecode);
                        }
                    }
                }
            }
        }

        // Clean up malformed node_id prior to creating one.
        db.execute(DELETE_NODEID, DatabaseParams::empty());
        let secret_key: SecretKey = SecretKey::generate(&mut rng);

        let key_string = hex::encode(secret_key.to_bytes());
        db.execute(INSERT_NODEID, DatabaseParams::single(DatabaseParam::String(key_string)));

        secret_key
    }

}
