use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::mocks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Mock {
    pub id: i32,
    pub is_enable: bool,
    pub request_method: String,
    pub request_url: String,
    pub response_file_path: String,
    pub response_status_code: i32,
    pub response_delay: i32,
    pub response_content_type: String,
}
