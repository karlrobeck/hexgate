mod routes {
    use axum::Router;

    pub struct HexgateRouter;

    impl HexgateRouter {
        pub fn build() -> Router {
            todo!("build the router using static methods")
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
