use std::{fmt::Display, str::FromStr};

use axum::{
    Router,
    handler::Handler,
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post, put},
};
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::debug;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::{AuthState, Roles, auth::roles::require_roles, metrics, validate_token};

struct Route<R> {
    method: &'static str,
    root_path: &'static str,
    relative_path: &'static str,
    required_roles: Option<R>,
}

impl<R> Display for Route<R>
where
    R: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {}{}",
            self.method, self.root_path, self.relative_path
        )?;

        if let Some(r) = &self.required_roles {
            write!(f, " (requires roles {r})")?;
        }

        Ok(())
    }
}

pub struct RouterBuilder<S, R> {
    inner: OpenApiRouter<S>,
    root_path: &'static str,
    routes: Vec<Route<R>>,
}

impl<S, R> RouterBuilder<S, R>
where
    S: Send + Sync + Clone + 'static,
    R: Roles,
    <R as FromStr>::Err: std::fmt::Debug,
{
    pub fn new(root_path: &'static str) -> Self {
        Self {
            inner: OpenApiRouter::new(),
            root_path,
            routes: Vec::new(),
        }
    }

    pub fn get<T, F>(mut self, path: &'static str, handler: F) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(path, get(handler));
        self.routes.push(Route {
            method: "GET",
            root_path: self.root_path,
            relative_path: path,
            required_roles: None,
        });
        self
    }

    pub fn role_protected_get<T, F>(mut self, path: &'static str, handler: F, roles: R) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(
            path,
            get(handler).layer(middleware::from_fn_with_state(
                roles.clone(),
                require_roles::<R>,
            )),
        );
        self.routes.push(Route {
            method: "GET",
            root_path: self.root_path,
            relative_path: path,
            required_roles: Some(roles),
        });
        self
    }

    pub fn post<T, F>(mut self, path: &'static str, handler: F) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(path, post(handler));
        self.routes.push(Route {
            method: "POST",
            root_path: self.root_path,
            relative_path: path,
            required_roles: None,
        });
        self
    }

    pub fn role_protected_post<T, F>(mut self, path: &'static str, handler: F, roles: R) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(
            path,
            post(handler).layer(middleware::from_fn_with_state(
                roles.clone(),
                require_roles::<R>,
            )),
        );
        self.routes.push(Route {
            method: "POST",
            root_path: self.root_path,
            relative_path: path,
            required_roles: Some(roles),
        });
        self
    }

    pub fn put<T, F>(mut self, path: &'static str, handler: F) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(path, put(handler));
        self.routes.push(Route {
            method: "PUT",
            root_path: self.root_path,
            relative_path: path,
            required_roles: None,
        });
        self
    }

    pub fn role_protected_put<T, F>(mut self, path: &'static str, handler: F, roles: R) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(
            path,
            put(handler).layer(middleware::from_fn_with_state(
                roles.clone(),
                require_roles::<R>,
            )),
        );
        self.routes.push(Route {
            method: "PUT",
            root_path: self.root_path,
            relative_path: path,
            required_roles: Some(roles),
        });
        self
    }

    pub fn patch<T, F>(mut self, path: &'static str, handler: F) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(path, patch(handler));
        self.routes.push(Route {
            method: "PATCH",
            root_path: self.root_path,
            relative_path: path,
            required_roles: None,
        });
        self
    }

    pub fn role_protected_patch<T, F>(mut self, path: &'static str, handler: F, roles: R) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(
            path,
            patch(handler).layer(middleware::from_fn_with_state(
                roles.clone(),
                require_roles::<R>,
            )),
        );
        self.routes.push(Route {
            method: "PATCH",
            root_path: self.root_path,
            relative_path: path,
            required_roles: Some(roles),
        });
        self
    }

    pub fn delete<T, F>(mut self, path: &'static str, handler: F) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(path, delete(handler));
        self.routes.push(Route {
            method: "DELETE",
            root_path: self.root_path,
            relative_path: path,
            required_roles: None,
        });
        self
    }

    pub fn role_protected_delete<T, F>(mut self, path: &'static str, handler: F, roles: R) -> Self
    where
        F: Handler<T, S>,
        T: 'static,
    {
        self.inner = self.inner.route(
            path,
            delete(handler).layer(middleware::from_fn_with_state(
                roles.clone(),
                require_roles::<R>,
            )),
        );
        self.routes.push(Route {
            method: "DELETE",
            root_path: self.root_path,
            relative_path: path,
            required_roles: Some(roles),
        });
        self
    }

    pub fn build_no_metrics(self, app_state: S, auth_state: AuthState, api_doc: OpenApi) -> Router {
        self.log_routes();
        let main_router = self.inner
            .route("/metrics", get(|| async { (StatusCode::SERVICE_UNAVAILABLE, "Metrics endpoint is disabled. Metrics must be enabled and the service restarted")}));
        build::<S, R>(self.root_path, main_router, app_state, auth_state, api_doc)
    }

    pub fn build_with_metrics(
        self,
        app_state: S,
        auth_state: AuthState,
        api_doc: OpenApi,
        metrics_handle: PrometheusHandle,
    ) -> Router {
        self.log_routes();

        let main_router = self
            .inner
            .route("/metrics", get(|| async move { metrics_handle.render() }))
            .route_layer(middleware::from_fn(metrics::track_http));

        build::<S, R>(self.root_path, main_router, app_state, auth_state, api_doc)
    }

    fn log_routes(&self) {
        for route in &self.routes {
            debug!("Building route - {route}")
        }
    }
}

fn build<S, R>(
    root_path: &'static str,
    main_router: OpenApiRouter<S>,
    app_state: S,
    auth_state: AuthState,
    api_doc: OpenApi,
) -> Router
where
    S: Send + Sync + Clone + 'static,
    R: Roles,
    <R as FromStr>::Err: std::fmt::Debug,
{
    let main_routes = OpenApiRouter::new()
        .nest(root_path, main_router)
        .layer(middleware::from_fn_with_state(
            auth_state,
            validate_token::<R>,
        ))
        // TODO metrics
        .with_state(app_state);
    let (router, api) = OpenApiRouter::with_openapi(api_doc)
        .merge(main_routes)
        .split_for_parts();

    router.merge(
        SwaggerUi::new(format!("{}/swagger-ui", root_path))
            .url(format!("{}/api-docs/openapi.json", root_path), api),
    )
}
