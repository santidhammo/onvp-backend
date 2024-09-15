CREATE TABLE members (
    id SERIAL PRIMARY KEY,
    member_details_id INT NOT NULL,
    member_address_details_id INT NOT NULL,
    musical_instrument_id INT NULL,
    picture_asset_id VARCHAR NULL,
    activated BOOLEAN NOT NULL DEFAULT FALSE,
    creation_time TIMESTAMP NOT NULL,
    activation_string VARCHAR NOT NULL,
    activation_time TIMESTAMP NOT NULL,
    -- If set, other members are allowed to see the details of the member. The member should have given written
    -- permission in accordance with the Digital Privacy Information Act (DPIA). The default value therefore is
    -- set to false, to make sure that a conscientious decision is made.
    allow_privacy_info_sharing BOOLEAN NOT NULL DEFAULT FALSE,
    nonce VARCHAR NOT NULL,
    CONSTRAINT fk_member_details FOREIGN KEY (member_details_id) REFERENCES member_details(id),
    CONSTRAINT fk_member_address_details FOREIGN KEY (member_address_details_id) REFERENCES member_address_details(id),
    CONSTRAINT fk_musical_instrument FOREIGN KEY (musical_instrument_id) REFERENCES musical_instruments(id)
);
