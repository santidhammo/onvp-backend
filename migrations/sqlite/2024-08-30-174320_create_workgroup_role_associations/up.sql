CREATE TABLE workgroup_role_associations (
    workgroup_id INT NOT NULL,
    system_role INT NOT NULL,
    PRIMARY KEY(workgroup_id, system_role),
    CONSTRAINT fk_workgroup_role_workgroup FOREIGN KEY (workgroup_id) REFERENCES workgroups(id) ON DELETE CASCADE
);