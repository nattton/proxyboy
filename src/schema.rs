// @generated automatically by Diesel CLI.

diesel::table! {
    logs (id) {
        id -> Integer,
        request_method -> Text,
        request_url -> Text,
        request_params -> Text,
        request_body -> Text,
        request_headers -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    mocks (id) {
        id -> Integer,
        name -> Text,
        is_enable -> Bool,
        request_method -> Text,
        request_url -> Text,
        response_file_path -> Text,
        response_status_code -> Integer,
        response_delay -> Integer,
        response_content_type -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    logs,
    mocks,
);
