use crate::errors::{Error, Result};
use boon::{Compiler, Schemas};
use colored::Color;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

pub trait Route {
    fn path(&self) -> &str;
    fn view(&self) -> &str;
    fn props(&self) -> Value {
        json!({})
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    manifest: ManifestContent,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum ManifestContent {
    RootView(RootView),
    RoutesDefinition(RoutesDefinition),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RootView {
    root_view: String,
}

impl Route for RootView {
    fn path(&self) -> &str {
        "/"
    }
    fn view(&self) -> &str {
        &self.root_view
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutesDefinition {
    lenra_routes: Option<Vec<LenraRoute>>,
    json_routes: Option<Vec<JsonRoute>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LenraRoute {
    path: String,
    view: ViewComponent,
}

impl Route for LenraRoute {
    fn path(&self) -> &str {
        &self.path
    }
    fn view(&self) -> &str {
        &self.view.name
    }
    fn props(&self) -> Value {
        self.view.props.clone().unwrap_or(json!({}))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct JsonRoute {
    path: String,
    view: String,
}

impl Route for JsonRoute {
    fn path(&self) -> &str {
        &self.path
    }
    fn view(&self) -> &str {
        &self.view
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ViewComponent {
    name: String,
    props: Option<Value>,
}

pub fn check_app() -> Result<CheckerLevel> {
    // TODO: load schema from url
    // let path = PathBuf::from("../../schemas/manifest.json");
    // let file = fs::File::open(path.clone()).map_err(|err| Error::OpenFile(err, path.clone()))?;
    // load manifest JSON Schema
    // let schema = include_str!("../schemas/manifest.json");
    // let schema = serde_json::from_str(schema).map_err(Error::from)?;

    // load manifest
    let manifest = load_check_schema(json!({}), "./schemas/manifest.json")?;
    let manifest: Manifest = serde_json::from_value(manifest).map_err(Error::from)?;

    let mut results: Vec<CheckerLevel> = match manifest.manifest {
        ManifestContent::RootView(root_view) => vec![check_route(&root_view)?],
        ManifestContent::RoutesDefinition(routes_def) => {
            let mut routes_results = vec![];
            if let Some(routes) = routes_def.lenra_routes {
                routes.iter().for_each(|route| {
                    routes_results.push(check_route(route).unwrap());
                });
            }
            if let Some(routes) = routes_def.json_routes {
                routes.iter().for_each(|route| {
                    routes_results.push(check_route(route).unwrap());
                });
            }
            routes_results
        }
    };

    Ok(CheckerLevel::Ok)
}

pub fn check_route(route: &dyn Route) -> Result<CheckerLevel> {
    let request = json!({
        "view": route.view(),
        "data": [],
        "props":route.props(),
    });
    let result = load_check_schema(request, "./schemas/view_result.json")?;
    println!("view result: {:?}", result);
    Ok(CheckerLevel::Ok)
}

fn load_check_schema(request: Value, schema_path: &str) -> Result<Value> {
    let mut schemas = Schemas::new(); // container for compiled schemas
    let mut compiler = Compiler::new();
    compiler.set_default_draft(boon::Draft::V2020_12);

    let json = call_app(request).map_err(Error::from)?;

    let sch_index = compiler
        .compile(schema_path, &mut schemas)
        .map_err(|error| {
            Error::Custom(format!(
                "Error while compiling the schema[{}]: {}",
                schema_path, error
            ))
        })?;

    let result = schemas.validate(&json, sch_index);

    if let Err(error) = result {
        println!("error: {}", error);
        // return Ok(CheckerLevel::Error);
    }

    Ok(json)
}

pub fn call_app<T: DeserializeOwned>(request: Value) -> Result<T> {
    ureq::post("http://localhost:8080")
        .send_json(request)
        .map_err(Error::from)?
        .into_json()
        .map_err(Error::from)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CheckerLevel {
    Ok,
    Warning,
    Error,
}

impl CheckerLevel {
    fn color(&self) -> Color {
        match self {
            CheckerLevel::Ok => Color::Green,
            CheckerLevel::Warning => Color::Yellow,
            CheckerLevel::Error => Color::Red,
        }
    }
}
