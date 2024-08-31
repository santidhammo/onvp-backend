CREATE TABLE page_access_policies (
    page_id INT NOT NULL,
    system_role INT NOT NULL,
    PRIMARY KEY (page_id, system_role),
    CONSTRAINT fk_page_access_policy_page FOREIGN KEY (page_id) REFERENCES pages(id) ON DELETE CASCADE
);