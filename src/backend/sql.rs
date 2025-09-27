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
    DELETE * FROM NodeID;
";
