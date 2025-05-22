/*
 *  ONVP Backend - Backend API provider for the ONVP website
 *
 * Copyright (c) 2025.  Sjoerd van Leent
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

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
    -- permission in accordance with the GDPR.
    -- it is set to false initially, making sure that a conscientious decision is made.
    allow_privacy_info_sharing BOOLEAN NOT NULL DEFAULT FALSE,
    nonce VARCHAR NOT NULL,
    CONSTRAINT fk_member_details FOREIGN KEY (member_details_id) REFERENCES member_details(id),
    CONSTRAINT fk_member_address_details FOREIGN KEY (member_address_details_id) REFERENCES member_address_details(id),
    CONSTRAINT fk_musical_instrument FOREIGN KEY (musical_instrument_id) REFERENCES musical_instruments(id)
);
