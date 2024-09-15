CREATE TABLE pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_asset VARCHAR NOT NULL UNIQUE,
    parent_id INT NULL,
    icon_asset VARCHAR NULL,
    event_date DATE NULL,
    etag VARCHAR NOT NULL,
    CONSTRAINT fk_page_parent FOREIGN KEY (parent_id) REFERENCES pages(id) ON DELETE SET NULL
);