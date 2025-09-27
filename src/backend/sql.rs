
// NODE ID //

pub const CREATE_NODEID_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS NodeID (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        secret TEXT NOT NULL,
        public TEXT NOT NULL
    );
";

pub const INSERT_NODEID: &str = "
    INSERT INTO NodeID
    VALUES(null, ?, ?)
";

pub const SELECT_NODEID: &str = "
    SELECT secret, public FROM NodeID;
";

pub const DELETE_NODEID: &str = "
    DELETE FROM NodeID;
";

// CONTACTS //

pub const CREATE_CONTACTS_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS Contacts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        node_id TEXT NOT NULL,
        username: TEXT NOT NULL
    );
";

pub const INSERT_CONTACT: &str = "
    INSERT INTO Contacts
    VALUES(null, ?, ?)
";

pub const DELETE_CONTACT: &str = "
    DELETE FROM Contacts WHERE node_id = ?;
";

pub const SELECT_ALL_CONTACTS: &str = "
    SELECT node_id, username FROM Contacts;
";

// USERNAME //
pub const CREATE_USERNAME_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS Username (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        username TEXT NOT NULL
    );
";

pub const INSERT_USERNAME: &str = "
    INSERT INTO Username
    VALUES(null, ?)
";

pub const SELECT_USERNAME: &str = "
    SELECT username FROM Username;
";
