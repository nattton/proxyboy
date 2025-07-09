use crate::schema::mocks::dsl::*;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::mocks)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Mock {
    pub id: i32,
    pub name: String,
    pub is_enable: bool,
    pub request_method: String,
    pub request_url: String,
    pub response_file_path: String,
    pub response_status_code: i32,
    pub response_delay: i32,
    pub response_content_type: String,
}

impl Mock {
    pub fn find_by_method_and_url(
        method: &str,
        url: &str,
        conn: &mut SqliteConnection,
    ) -> Option<Mock> {
        let mock = mocks
            .filter(is_enable.eq(true))
            .filter(
                request_method
                    .like(format!("%{}%", method))
                    .or(request_method.eq("*")),
            )
            .filter(request_url.like(format!("%{}", url)))
            .first::<Mock>(conn)
            .optional()
            .expect("Error loading mocks");
        mock
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::mocks)]
pub struct InsertMock {
    pub name: String,
    pub is_enable: bool,
    pub request_method: String,
    pub request_url: String,
    pub response_file_path: String,
    pub response_status_code: i32,
    pub response_delay: i32,
    pub response_content_type: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::logs)]
pub struct InsertLog {
    pub request_method: String,
    pub request_url: String,
    pub request_params: String,
    pub request_body: String,
    pub request_headers: String,
}
