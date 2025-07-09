CREATE TABLE logs (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    request_method TEXT NOT NULL,
    request_url TEXT NOT NULL,
    request_params TEXT NOT NULL,
    request_body TEXT NOT NULL,
    request_headers TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);