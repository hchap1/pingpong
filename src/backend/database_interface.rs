use iroh::SecretKey;

use super::database::{DataLink, DatabaseParam, DatabaseParams};
use super::sql::*;

use rand::rngs::OsRng;

pub struct DatabaseInterface;

impl DatabaseInterface {

    pub fn make_tables_nonblocking(db: DataLink) {
        let _ = db.execute(CREATE_NODEID_TABLE, DatabaseParams::empty());
    }

    pub async fn get_node_id_blocking(db: DataLink) -> SecretKey {

        let mut rng = OsRng;
        
        if let Ok(results) = db.query_map(SELECT_NODEID, DatabaseParams::empty()).await {
            if let Some(row) = results.first() {
                if let Some(node_id) = row.first() {
                    if let Ok(secret_key) = hex::decode(node_id.string()) {
                        if let Ok(bytecode) = &secret_key.try_into() {
                            return SecretKey::from_bytes(bytecode);
                        }
                    }
                }
            }
        }

        // Clean up malformed node_id prior to creating one.
        let _ = db.execute_and_wait(DELETE_NODEID, DatabaseParams::empty()).await;
        let secret_key: SecretKey = SecretKey::generate(&mut rng);

        let key_string = hex::encode(secret_key.to_bytes());
        let _ = db.execute_and_wait(INSERT_NODEID, DatabaseParams::single(DatabaseParam::String(key_string))).await;

        secret_key
    }

}
