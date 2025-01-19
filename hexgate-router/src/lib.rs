mod routes {
    use axum::{
        Router,
        routing::{delete, get, patch, post, put},
    };

    pub struct HexgateRouter;

    impl HexgateRouter {
        pub fn build() -> Router {
            Router::new()
                .route("/:schema/:table", post(HexgateRouter::insert_route))
                .route("/:schema/:table", get(HexgateRouter::select_route))
                .route("/:schema/:table", patch(HexgateRouter::update_route))
                .route("/:schema/:table", put(HexgateRouter::update_route))
                .route("/:schema/:table", delete(HexgateRouter::delete_route))
        }

        // select,update,insert,delete, execute function
        pub async fn select_route() {
            todo!("implement select operation for this route")
        }

        pub async fn update_route() {
            todo!("implement update operation for this route. use transaction here")
        }

        pub async fn insert_route() {
            todo!("implement insert operation for this route. use transaction here")
        }

        pub async fn delete_route() {
            todo!("implement delete operation for this route. use transaction here")
        }

        pub async fn execute_function() {
            todo!("implement database function execution for this route. use transaction here")
        }
    }
}
