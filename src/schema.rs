// @generated automatically by Diesel CLI.

diesel::table! {
    mocks (id) {
        id -> Integer,
        is_enable -> Bool,
        request_method -> Text,
        request_url -> Text,
        response_file_path -> Text,
        response_status_code -> Integer,
        response_delay -> Integer,
        response_content_type -> Text,
    }
}
