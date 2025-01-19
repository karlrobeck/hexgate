mod routes {
    use std::collections::BTreeMap;

    use axum::{
        Router,
        extract::{Path, Query, State},
        routing::{delete, get, patch, post, put},
    };
    use serde::{Deserialize, Serialize};
    use sqlx::{Pool, Postgres};

    #[derive(Clone)]
    pub struct HexgateRouter {
        db: Pool<Postgres>,
    }

    #[derive(Deserialize, Serialize)]
    pub struct SQLSchemaTable {
        schema: String,
        table: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct SQLFunction {
        schema: String,
        name: String,
    }

    #[derive(Deserialize, Serialize)]
    pub struct SQLOperation {
        limit: i32,
        filter: String, // TODO: implement this properly for parsing
        sort: String,   // TODO: implement this properly for parsing
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
                    post(HexgateRouter::execute_function),
                )
                .with_state(self)
        }

        // select,update,insert,delete, execute function
        pub async fn select_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Query(query): Query<SQLOperation>,
        ) {
            todo!("implement select operation for this route")
        }

        pub async fn update_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Query(query): Query<SQLOperation>,
        ) {
            todo!("implement update operation for this route. use transaction here")
        }

        pub async fn insert_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Query(query): Query<SQLOperation>,
        ) {
            todo!("implement insert operation for this route. use transaction here")
        }

        pub async fn delete_route(
            State(state): State<HexgateRouter>,
            Path(path): Path<SQLSchemaTable>,
            Query(query): Query<SQLOperation>,
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
