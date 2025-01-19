mod sql {
    use sea_query::{Alias, Query, SelectStatement, UpdateStatement};
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
        distinct_on: Option<Vec<String>>,
    }

    impl SQLOperation {
        pub fn build_select(&self, schema: Alias, table: Alias) -> SelectStatement {
            let mut sql_query = Query::select().from((schema, table)).to_owned();
            let sql_query = match self.limit {
                Some(limit) => sql_query.limit(limit).to_owned(),
                None => sql_query,
            };
            sql_query
        }
        pub fn build_insert(&self, schema: Alias, table: Alias) {
            let mut sql_query = Query::insert()
                .into_table((schema, table))
                .returning_all()
                .to_owned();
        }
    }

    pub struct ToValue(pub Value);

    impl Into<UpdateStatement> for ToValue {
        fn into(self) -> UpdateStatement {
            unimplemented!()
        }
    }

    #[cfg(test)]
    mod test {
        use axum::{extract::Query, http::Uri};
        use sea_query::{Alias, Asterisk, PostgresQueryBuilder, Query as DBQuery};

        use super::SQLOperation;

        #[test]
        fn test_limit_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");
            let query: Query<SQLOperation> = Query::try_from_uri(&Uri::from_static(
                "http://example.com/sample/table/?limit=10",
            ))
            .unwrap();

            let sql_query = DBQuery::select()
                .from((schema_name, table_name))
                .column(Asterisk)
                .limit(query.limit.unwrap())
                .to_owned();

            assert_eq!(
                sql_query.to_string(PostgresQueryBuilder),
                r#"SELECT * FROM "sample"."table" LIMIT 10"#
            );
        }

        #[test]
        fn test_empty_columns_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");

            let query: Query<SQLOperation> =
                Query::try_from_uri(&Uri::from_static("http://example.com/sample/table/")).unwrap();

            let mut sql_query = DBQuery::select().from((schema_name, table_name)).to_owned();

            let sql_query = match &query.columns {
                Some(columns) => sql_query
                    .columns(columns.split(",").map(Alias::new).collect::<Vec<_>>())
                    .to_owned(),
                None => sql_query.column(Asterisk).to_owned(),
            };

            assert_eq!(
                sql_query.to_string(PostgresQueryBuilder),
                r#"SELECT * FROM "sample"."table""#
            )
        }

        #[test]
        fn test_with_columns_query() {
            let schema_name = Alias::new("sample");
            let table_name = Alias::new("table");

            let query: Query<SQLOperation> = Query::try_from_uri(&Uri::from_static(
                "http://example.com/sample/table/?columns=id,name,password",
            ))
            .unwrap();

            let mut sql_query = DBQuery::select().from((schema_name, table_name)).to_owned();

            let sql_query = match &query.columns {
                Some(columns) => sql_query
                    .columns(columns.split(",").map(Alias::new).collect::<Vec<_>>())
                    .to_owned(),
                None => sql_query.column(Asterisk).to_owned(),
            };

            assert_eq!(
                sql_query.to_string(PostgresQueryBuilder),
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

            let mut sql_query = DBQuery::select()
                .from((schema_name, table_name))
                .column(Asterisk)
                .to_owned();

            let sql_query = match query.distinct {
                Some(_) => sql_query.distinct().to_owned(),
                None => sql_query,
            };

            assert_eq!(
                sql_query.to_string(PostgresQueryBuilder),
                r#"SELECT DISTINCT * FROM "sample"."table""#
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

    use crate::sql::{SQLFunction, SQLOperation, SQLSchemaTable, ToValue};

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

            let sql = query
                .build_select(schema_name, table_name)
                .to_owned()
                .to_string(PostgresQueryBuilder);

            // execute the query
            let result = sqlx::query(&sql).fetch_all(&state.db).await;

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
