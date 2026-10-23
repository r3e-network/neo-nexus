//! The route table and the documents that promise endpoints.
//!
//! The router registers paths as literals; the documentation names the same
//! paths in prose. Comparing the two textually is unreliable — a comment
//! between the path and the handler defeats a regular expression, and a
//! quoted string anywhere in a line looks like a path — so the route table is
//! read from the syntax tree and the documents are read with a YAML parser.

use std::collections::BTreeSet;

use syn::visit::Visit;
use syn::{Expr, ExprMethodCall, Lit};

/// A route the web layer serves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServedRoute {
    /// The HTTP method, or `None` when the router does not name one.
    pub method: Option<String>,
    /// The path, as the router spells it.
    pub path: String,
}

/// An endpoint a document promises.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedEndpoint {
    /// The HTTP method.
    pub method: String,
    /// The path, as the document spells it.
    pub path: String,
    /// Where the promise is written, for a report.
    pub origin: String,
}

/// The HTTP methods the router and the documents may name.
pub fn http_method(name: &str) -> Option<&'static str> {
    match name.to_ascii_lowercase().as_str() {
        "get" => Some("GET"),
        "post" => Some("POST"),
        "put" => Some("PUT"),
        "delete" => Some("DELETE"),
        "patch" => Some("PATCH"),
        "head" => Some("HEAD"),
        "options" => Some("OPTIONS"),
        "trace" => Some("TRACE"),
        _ => None,
    }
}

/// Whether a served route answers a documented endpoint.
///
/// A route that names no method answers every method on that path.
pub fn route_answers(route: &ServedRoute, endpoint: &DocumentedEndpoint) -> bool {
    if let Some(method) = &route.method {
        if method != &endpoint.method {
            return false;
        }
    }
    normalize(&route.path) == normalize(&endpoint.path)
}

/// The same path, spelled the same way.
///
/// `/api/nodes/{node_id}/metrics` and `/api/nodes/{id}/metrics` address the
/// same route; only one spelling can be right in both files.
pub fn normalize(path: &str) -> String {
    let trimmed = path.trim();
    let trimmed = if trimmed.len() > 1 {
        trimmed.trim_end_matches('/')
    } else {
        trimmed
    };
    trimmed
        .split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                "{}".to_string()
            } else {
                segment.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Collect every route the source registers.
pub fn collect_routes(source: &str) -> Vec<ServedRoute> {
    let file = match syn::parse_file(source) {
        Ok(file) => file,
        Err(_) => return Vec::new(),
    };
    let mut visitor = RouteVisitor { routes: Vec::new() };
    visitor.visit_file(&file);
    visitor.routes
}

/// Reads `.route(path, method_router)` calls out of the syntax tree.
#[derive(Default)]
struct RouteVisitor {
    routes: Vec<ServedRoute>,
}

impl<'ast> Visit<'ast> for RouteVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if node.method == "route" {
            let mut arguments = node.args.iter();
            if let Some(Expr::Lit(literal)) = arguments.next() {
                if let Lit::Str(path) = &literal.lit {
                    let methods = arguments.next().map(collect_methods).unwrap_or_default();
                    if methods.is_empty() {
                        self.routes.push(ServedRoute {
                            method: None,
                            path: path.value(),
                        });
                    } else {
                        for method in methods {
                            self.routes.push(ServedRoute {
                                method: Some(method),
                                path: path.value(),
                            });
                        }
                    }
                }
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// The HTTP methods a method router answers.
///
/// A method router is a call to `get`, `post` and friends, sometimes with a
/// layer wrapped around it: `get(handler).route_layer(layer)`.
fn collect_methods(expression: &Expr) -> Vec<String> {
    let mut methods = Vec::new();
    collect_methods_into(expression, &mut methods);
    methods
}

fn collect_methods_into(expression: &Expr, methods: &mut Vec<String>) {
    match expression {
        Expr::Call(call) => {
            if let Expr::Path(path) = &*call.func {
                if let Some(segment) = path.path.segments.last() {
                    let name = segment.ident.to_string();
                    if let Some(method) = http_method(&name) {
                        methods.push(method.to_string());
                    }
                }
            }
        }
        Expr::MethodCall(call) => {
            let name = call.method.to_string();
            if let Some(method) = http_method(&name) {
                methods.push(method.to_string());
            }
            collect_methods_into(&call.receiver, methods);
        }
        Expr::Group(group) => collect_methods_into(&group.expr, methods),
        Expr::Paren(paren) => collect_methods_into(&paren.expr, methods),
        _ => {}
    }
}

/// The endpoints the OpenAPI document promises.
pub fn openapi_endpoints(source: &str) -> anyhow::Result<Vec<DocumentedEndpoint>> {
    let document: serde_yaml::Value = serde_yaml::from_str(source)?;
    let paths = document
        .get("paths")
        .and_then(serde_yaml::Value::as_mapping)
        .ok_or_else(|| anyhow::anyhow!("the OpenAPI document has no `paths` mapping"))?;

    let mut endpoints = Vec::new();
    for (path, item) in paths {
        let Some(path) = path.as_str() else {
            continue;
        };
        let Some(operations) = item.as_mapping() else {
            continue;
        };
        for key in operations.keys() {
            let Some(key) = key.as_str() else {
                continue;
            };
            // A path item also carries `parameters` and `summary`; only the
            // method keys are endpoints.
            if let Some(method) = http_method(key) {
                endpoints.push(DocumentedEndpoint {
                    method: method.to_string(),
                    path: path.to_string(),
                    origin: "docs/openapi.yaml".to_string(),
                });
            }
        }
    }
    Ok(endpoints)
}

/// The endpoints the agent handbook promises.
///
/// The handbook is prose, but its endpoint sections are headings of the form
/// `### GET /api/fleet`. Headings that are not endpoints — `### Session
/// Cookie Authentication` — are skipped by the method filter.
pub fn handbook_endpoints(source: &str, origin: &str) -> Vec<DocumentedEndpoint> {
    let mut endpoints = Vec::new();
    for (index, line) in source.lines().enumerate() {
        let Some(heading) = line.trim().strip_prefix("### ") else {
            continue;
        };
        let mut words = heading.split_whitespace();
        let Some(method) = words.next() else {
            continue;
        };
        let Some(path) = words.next() else {
            continue;
        };
        if !path.starts_with('/') {
            continue;
        }
        let Some(method) = http_method(method) else {
            continue;
        };
        endpoints.push(DocumentedEndpoint {
            method: method.to_string(),
            path: path.to_string(),
            origin: format!("{origin}:{}", index + 1),
        });
    }
    endpoints
}

/// The documented endpoints that no route answers.
pub fn undocumented_endpoints(
    documented: &[DocumentedEndpoint],
    served: &[ServedRoute],
) -> Vec<DocumentedEndpoint> {
    let mut missing = Vec::new();
    let mut seen = BTreeSet::new();
    for endpoint in documented {
        if served.iter().any(|route| route_answers(route, endpoint)) {
            continue;
        }
        if seen.insert((endpoint.method.clone(), normalize(&endpoint.path))) {
            missing.push(endpoint.clone());
        }
    }
    missing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_route_is_read_from_the_syntax_tree() {
        let source = "fn build() { Router::new().route(\"/api/fleet\", get(handler)); }";
        let routes = collect_routes(source);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method.as_deref(), Some("GET"));
        assert_eq!(routes[0].path, "/api/fleet");
    }

    #[test]
    fn a_comment_between_the_path_and_the_handler_does_not_hide_the_route() {
        let source = "fn build() {\n    Router::new().route(\n        \"/plugins/install\",\n        // The handler streams the body itself.\n        post(install),\n    );\n}";
        let routes = collect_routes(source);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/plugins/install");
        assert_eq!(routes[0].method.as_deref(), Some("POST"));
    }

    #[test]
    fn a_layered_method_router_keeps_its_method() {
        let source =
            "fn build() { Router::new().route(\"/api/logs\", get(logs).route_layer(layer)); }";
        let routes = collect_routes(source);
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method.as_deref(), Some("GET"));
    }

    #[test]
    fn a_quoted_string_that_is_not_a_route_is_ignored() {
        let source = "fn f() { let _ = \"/login\"; let _ = \"/api/\"; }";
        assert!(collect_routes(source).is_empty());
    }

    #[test]
    fn a_route_without_a_literal_path_is_ignored() {
        let source = "fn f() { let route = self.config.route(path, query); }";
        assert!(collect_routes(source).is_empty());
    }

    #[test]
    fn a_handbook_heading_that_is_not_an_endpoint_is_skipped() {
        let source = "### Session Cookie Authentication\n\n### GET /api/fleet\n";
        let endpoints = handbook_endpoints(source, "docs/AGENT_API.md");
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].method, "GET");
        assert_eq!(endpoints[0].path, "/api/fleet");
    }

    #[test]
    fn the_openapi_document_yields_its_method_and_path() {
        let source = "openapi: 3.0.0\npaths:\n  /api/fleet:\n    get:\n      responses: {}\n  /api/jobs:\n    post:\n      responses: {}\n";
        let endpoints = openapi_endpoints(source).expect("the document parses");
        assert_eq!(endpoints.len(), 2);
        assert!(endpoints
            .iter()
            .any(|endpoint| endpoint.method == "GET" && endpoint.path == "/api/fleet"));
        assert!(endpoints
            .iter()
            .any(|endpoint| endpoint.method == "POST" && endpoint.path == "/api/jobs"));
    }

    #[test]
    fn a_parameter_name_does_not_change_the_route() {
        let route = ServedRoute {
            method: Some("GET".to_string()),
            path: "/api/nodes/{node_id}/metrics".to_string(),
        };
        let endpoint = DocumentedEndpoint {
            method: "GET".to_string(),
            path: "/api/nodes/{id}/metrics".to_string(),
            origin: "docs/openapi.yaml".to_string(),
        };
        assert!(route_answers(&route, &endpoint));
    }

    #[test]
    fn a_documented_endpoint_without_a_route_is_missing() {
        let documented = vec![DocumentedEndpoint {
            method: "GET".to_string(),
            path: "/api/fleet".to_string(),
            origin: "docs/openapi.yaml".to_string(),
        }];
        let served = vec![ServedRoute {
            method: Some("POST".to_string()),
            path: "/api/fleet".to_string(),
        }];
        assert_eq!(undocumented_endpoints(&documented, &served).len(), 1);
    }

    #[test]
    fn a_route_that_names_no_method_answers_every_method() {
        let served = vec![ServedRoute {
            method: None,
            path: "/api/fleet".to_string(),
        }];
        for method in ["GET", "POST"] {
            let documented = vec![DocumentedEndpoint {
                method: method.to_string(),
                path: "/api/fleet".to_string(),
                origin: "docs/openapi.yaml".to_string(),
            }];
            assert!(undocumented_endpoints(&documented, &served).is_empty());
        }
    }
}
