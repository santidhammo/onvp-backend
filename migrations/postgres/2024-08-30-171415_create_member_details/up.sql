CREATE TABLE member_details (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email_address VARCHAR NOT NULL,
    phone_number VARCHAR NOT NULL,
    CHECK (phone_number ~ '^\+[0-9]+$')
)
