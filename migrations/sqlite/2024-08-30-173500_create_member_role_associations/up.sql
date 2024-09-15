CREATE TABLE member_role_associations (
    member_id INT NOT NULL,
    system_role INT NOT NULL,
    PRIMARY KEY(member_id, system_role),
    CONSTRAINT fk_member_role_member FOREIGN KEY (member_id) REFERENCES members(id) ON DELETE CASCADE
);