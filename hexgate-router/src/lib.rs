pub mod sql {
    use sea_query::{
        Alias, Asterisk, PostgresQueryBuilder, Query, QueryStatementWriter, SelectStatement,
        UpdateStatement,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::Value;

    #[derive(Deserialize, Serialize)]
    pub struct SQLSchemaTable {
        pub schema: String,
        pub table: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct SQLFunction {
        schema: String,
        name: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct SQLOperation {
        limit: Option<u64>,
        filter: Option<String>, // TODO: implement this properly for parsing
        sort: Option<String>,   // TODO: implement this properly for parsing
        distinct: Option<bool>,
        columns: Option<String>,
        distinct_on: Option<String>,
    }

    pub struct SelectSQL {
        statement: SelectStatement,
        operation: SQLOperation,
    }

    impl SelectSQL {
        pub fn new(schema: Alias, table: Alias, operation: SQLOperation) -> Self {
            Self {
                statement: SelectStatement::new().from((schema, table)).to_owned(),
                operation,
            }
        }

        fn match_columns(mut self) -> Self {
            match &self.operation.columns {
                Some(columns) => {
                    let columns = columns.split(",").map(Alias::new).collect::<Vec<_>>();
                    self.statement = self.statement.columns(columns).to_owned();
                }
                None => {
                    self.statement = self.statement.column(Asterisk).to_owned();
                }
            };
            self
        }

        fn match_limit(mut self) -> Self {
            if let Some(limit) = &self.operation.limit {
                self.statement = self.statement.limit(limit.clone()).to_owned();
            }
            self
        }

        fn match_distinct(mut self) -> Self {
            if let Some(_) = &self.operation.distinct {
                self.statement = self.statement.distinct().to_owned();
            }
            self
        }

        fn match_distinct_on(mut self) -> Self {
            if let Some(distinct_on) = &self.operation.distinct_on {
                let distinct_on_columns =
                    distinct_on.split(",").map(Alias::new).collect::<Vec<_>>();
                self.statement = self.statement.distinct_on(distinct_on_columns).to_owned();
            }
            self
        }

        fn match_where(mut self) -> Self {
            self
        }

        pub fn build(mut self) -> String {
            self = self.match_columns();
            self = self.match_limit();
            self = self.match_distinct();
            self = self.match_distinct_on();
            self.statement.to_owned().to_string(PostgresQueryBuilder)
        }
    }

    #[cfg(test)]
    mod test {
        use axum::{extract::Query, http::Uri};
        use sea_query::Alias;

        use crate::sql::SelectSQL;

        use super::SQLOperation;

        #[test]
        fn test_limit_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");
            let query: Query<SQLOperation> = Query::try_from_uri(&Uri::from_static(
                "http://example.com/sample/table/?limit=10",
            ))
            .unwrap();
            let sql_query = SelectSQL::new(schema_name, table_name, query.0).build();
            assert_eq!(sql_query, r#"SELECT * FROM "sample"."table" LIMIT 10"#);
        }

        #[test]
        fn test_empty_columns_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");

            let query: Query<SQLOperation> =
                Query::try_from_uri(&Uri::from_static("http://example.com/sample/table/")).unwrap();

            let sql_query = SelectSQL::new(schema_name, table_name, query.0).build();

            assert_eq!(sql_query, r#"SELECT * FROM "sample"."table""#)
        }

        #[test]
        fn test_with_columns_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");

            let query: Query<SQLOperation> = Query::try_from_uri(&Uri::from_static(
                "http://example.com/sample/table/?columns=id,name,password",
            ))
            .unwrap();

            let sql_query = SelectSQL::new(schema_name, table_name, query.0).build();

            assert_eq!(
                sql_query,
                r#"SELECT "id", "name", "password" FROM "sample"."table""#
            )
        }

        #[test]
        fn test_distinct_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");

            let query: Query<SQLOperation> = Query::try_from_uri(&Uri::from_static(
                "http://example.com/sample/table/?distinct=true",
            ))
            .unwrap();

            let sql_query = SelectSQL::new(schema_name, table_name, query.0).build();

            assert_eq!(sql_query, r#"SELECT DISTINCT * FROM "sample"."table""#)
        }

        #[test]
        fn test_distinct_on_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");
            // distinct=true can be optional
            let query: Query<SQLOperation> = Query::try_from_uri(&Uri::from_static(
                "http://example.com/sample/table/?distinct=true&distinct_on=id,name,password&columns=id,name,password",
            ))
            .unwrap();

            let sql_query = SelectSQL::new(schema_name, table_name, query.0).build();

            assert_eq!(
                sql_query,
                r#"SELECT DISTINCT ON ("id", "name", "password") "id", "name", "password" FROM "sample"."table""#
            )
        }
    }
}

mod routes {

    use axum::{
        Json, Router,
        extract::{Path, Query, State},
        routing::{delete, get, patch, post, put},
    };
    use sea_query::{
        Alias, PostgresQueryBuilder, Query as DBQuery, QueryBuilder, SelectStatement,
        UpdateStatement,
    };
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use sqlx::{Pool, Postgres};

    use crate::sql::{SQLFunction, SQLOperation, SQLSchemaTable};

    #[derive(Clone)]
    pub struct HexgateRouter {
        db: Pool<Postgres>,
    }

    impl HexgateRouter {
        pub fn new(db: Pool<Postgres>) -> Self {
            Self { db }
        }

        pub fn build(self) -> Router {
            Router::new()
                .route("/:schema/:table", post(HexgateRouter::insert_route))
                .route("/:schema/:table", get(HexgateRouter::select_route))
                .route("/:schema/:table", patch(HexgateRouter::update_route))
                .route("/:schema/:table", put(HexgateRouter::update_route))
                .route("/:schema/:table", delete(HexgateRouter::delete_route))
                .route(
                    "/:schema/function/:name",
                    post(HexgateRouter::execute_function),
                )
                .route(
                    "/:schema/function/:name",
                    get(HexgateRouter::execute_function),
                )
                .with_state(self)
        }

        // select,update,insert,delete, execute function
        pub async fn select_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Query(query): Query<SQLOperation>,
        ) {
            let schema_name = Alias::new(path.schema);
            let table_name = Alias::new(path.table);

            todo!("implement select operation for this route")
        }

        pub async fn update_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Query(query): Query<SQLOperation>,
            Json(payload): Json<Value>,
        ) {
            todo!("implement update operation for this route. use transaction here")
        }

        pub async fn insert_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Json(payload): Json<Value>,
        ) {
            todo!("implement insert operation for this route. use transaction here")
        }

        pub async fn delete_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
        ) {
            todo!("implement delete operation for this route. use transaction here")
        }

        pub async fn execute_function(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLFunction>,
            Query(query): Query<serde_json::Value>, // key value pair json
        ) {
            todo!("implement database function execution for this route. use transaction here")
        }
    }
}
