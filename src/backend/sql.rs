
// NODE ID //

pub const CREATE_NODEID_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS NodeID (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        node_id TEXT NOT NULL
    );
";

pub const INSERT_NODEID: &str = "
    INSERT INTO NodeID
    VALUES(null, ?)
";

pub const SELECT_NODEID: &str = "
    SELECT node_id FROM NodeID;
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
