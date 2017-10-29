CREATE TABLE blobs (
    uuid TEXT NOT NULL PRIMARY KEY,
    owner VARCHAR(64) NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (owner) REFERENCES users(username)
)
