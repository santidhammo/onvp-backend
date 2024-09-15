CREATE TABLE member_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email_address VARCHAR NOT NULL,
    phone_number VARCHAR NOT NULL,
    CHECK (phone_number GLOB '+[0-9][0-9]*')
)
