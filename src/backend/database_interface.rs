use async_channel::Receiver;
use iroh::SecretKey;

use crate::networking::contact::Contact;

use super::database::{DataLink, DatabaseParam, DatabaseParams, ItemStream};
use super::sql::{CREATE_NODEID_TABLE, CREATE_USERNAME_TABLE, INSERT_USERNAME, SELECT_USERNAME};
use super::sql::SELECT_NODEID;
use super::sql::DELETE_NODEID;
use super::sql::CREATE_CONTACTS_TABLE;
use super::sql::SELECT_ALL_CONTACTS;
use super::sql::INSERT_CONTACT;
use super::sql::INSERT_NODEID;

use rand::rngs::OsRng;

pub struct DatabaseInterface;

impl DatabaseInterface {

    pub fn make_tables_nonblocking(db: DataLink) {
        let _ = db.execute(CREATE_NODEID_TABLE, DatabaseParams::empty());
        let _ = db.execute(CREATE_CONTACTS_TABLE, DatabaseParams::empty());
        let _ = db.execute(CREATE_USERNAME_TABLE, DatabaseParams::empty());
    }

    pub async fn get_node_id_blocking(db: DataLink) -> (SecretKey, SecretKey) {

        let mut rng = OsRng;
        
        if let Ok(results) = db.query_map(SELECT_NODEID, DatabaseParams::empty()).await {
            if let Some(row) = results.first() {
                if let (Some(a), Some(b)) = (row.first(), row.get(1)) {
                    if let (Ok(secret_key), Ok(node_id)) = (hex::decode(a.string()), hex::decode(b.string())) {
                        if let (Ok(bytecode_a), Ok(bytecode_b)) = (&secret_key.try_into(), &node_id.try_into()) {
                            return (SecretKey::from_bytes(bytecode_a), SecretKey::from_bytes(bytecode_b));
                        }
                    }
                }
            }
        }

        // Clean up malformed node_id prior to creating one.
        let _ = db.execute_and_wait(DELETE_NODEID, DatabaseParams::empty()).await;

        let secret_key_a: SecretKey = SecretKey::generate(&mut rng);
        let secret_key_b: SecretKey = SecretKey::generate(&mut rng);

        let key_string_a = hex::encode(secret_key_a.to_bytes());
        let key_string_b = hex::encode(secret_key_b.to_bytes());

        let _ = db.execute_and_wait(INSERT_NODEID, DatabaseParams::new(vec![
            DatabaseParam::String(key_string_a),
            DatabaseParam::String(key_string_b)
        ])).await;

        (secret_key_a, secret_key_b)
    }

    pub fn select_all_contacts(db: DataLink) -> Receiver<ItemStream> {
        db.query_stream(SELECT_ALL_CONTACTS, DatabaseParams::empty())
    }

    pub fn insert_contact(db: DataLink, contact: Contact) {
        let _ = db.execute(INSERT_CONTACT, contact.to_params());
    }

    pub fn select_username(db: DataLink) -> Option<String> {
        match db.query_blocking(&SELECT_USERNAME, DatabaseParams::empty()) {
            Ok(rows) => if let Some(first) = rows.first() { first.first().map(|p| p.string()) } else { None },
            Err(_) => None
        }
    }

    pub fn insert_username(db: DataLink, username: String) {
        let _ = db.execute(INSERT_USERNAME, DatabaseParams::single(DatabaseParam::String(username)));
    }
}
