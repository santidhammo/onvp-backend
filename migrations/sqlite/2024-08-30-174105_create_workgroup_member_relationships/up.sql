CREATE TABLE workgroup_member_relationships (
    workgroup_id INT NOT NULL,
    member_id INT NOT NULL,
    PRIMARY KEY (workgroup_id, member_id),
    CONSTRAINT fk_workgroup_member_workgroup FOREIGN KEY (workgroup_id) REFERENCES workgroups(id) ON DELETE CASCADE,
    CONSTRAINT fk_workgroup_member_member FOREIGN KEY (member_id) REFERENCES members(id) ON DELETE CASCADE
)