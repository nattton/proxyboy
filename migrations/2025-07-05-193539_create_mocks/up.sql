CREATE TABLE mocks (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    is_enable BOOLEAN NOT NULL DEFAULT 1,
    request_method TEXT NOT NULL DEFAULT '*',
    request_url TEXT NOT NULL,
    response_file_path TEXT NOT NULL,
    response_status_code INTEGER NOT NULL DEFAULT 200,
    response_delay INTEGER NOT NULL DEFAULT 0,
    response_content_type TEXT NOT NULL DEFAULT 'application/json'
);