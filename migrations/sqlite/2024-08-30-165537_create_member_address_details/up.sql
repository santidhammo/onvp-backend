CREATE TABLE member_address_details (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    street VARCHAR NOT NULL,
    house_number INT NOT NULL,
    house_number_postfix VARCHAR,
    postal_code VARCHAR NOT NULL,
    domicile VARCHAR NOT NULL
    CHECK (house_number > 0)
    CHECK (postal_code GLOB '[0-9][0-9][0-9][0-9][A-Z][A-Z]')
);