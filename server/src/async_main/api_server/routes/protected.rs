use std::time::Duration;

use axum::Router;
use axum::routing::get;

use crate::async_main::api_server::state::ApiServerSharedState;

pub fn make_routes(state: ApiServerSharedState) -> Router<ApiServerSharedState> {
	Router::new()
		.route("/protected", get(|| async {
			tokio::time::sleep(Duration::from_secs(2)).await;
			"Protected data"
		}))
		.with_state(state)
}