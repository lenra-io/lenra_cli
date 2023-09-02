use std::{
    fs::{self},
    path::PathBuf,
};

use crate::errors::{Error, Result};
use colored::Color;
use jsonschema::{Draft, JSONSchema};
use log::debug;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};

pub trait Route {
    fn path(&self) -> &str;
    fn view(&self) -> &str;
    fn props(&self) -> Value {
        json!({})
    }
    fn check(&self) -> Result<Vec<CheckerLevel>> {
        let request = json!({
            "view": self.view(),
            "data": [],
            "props":self.props(),
        });
        let result = load_check_schema(request, PathBuf::from("./schemas/view_result.json"))?;
        debug!("result: {:?}", result);
        Ok(vec![CheckerLevel::Ok])
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct ManifestResponse {
    manifest: Manifest,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Manifest {
    lenra: Option<Exposer<LenraRoute>>,
    json: Option<Exposer<JsonRoute>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Exposer<T: Route> {
    version: Option<String>,
    routes: Vec<T>,
}

impl<T: Route> Exposer<T> {
    pub fn check(&self) -> Result<Vec<CheckerLevel>> {
        let mut results = vec![];
        for route in &self.routes {
            route.check()?.iter().for_each(|r| results.push(r.clone()));
        }
        Ok(results)
    }
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

pub fn check_app() -> Result<Vec<CheckerLevel>> {
    // TODO: load schema from url

    // load manifest
    let manifest = load_check_schema(json!({}), PathBuf::from("./schemas/manifest.json"))?;
    let manifest: ManifestResponse = serde_json::from_value(manifest).map_err(Error::from)?;

    debug!("manifest: {:?}", manifest);

    let mut results: Vec<CheckerLevel> = vec![];
    if let Some(exposer) = manifest.manifest.lenra {
        results.append(&mut exposer.check()?);
    }
    if let Some(exposer) = manifest.manifest.json {
        results.append(&mut exposer.check()?);
    }

    Ok(results)
}

fn load_check_schema(request: Value, schema_path: PathBuf) -> Result<Value> {
    // TODO: load schema from schema_path
    let mut schema = serde_json::from_reader(
        fs::File::open(schema_path.clone())
            .map_err(|err| Error::OpenFile(err, schema_path.clone()))?,
    )?;
    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .map_err(|error| {
            Error::Custom(format!(
                "Error while compiling the schema[{:?}]: {}",
                schema_path, error
            ))
        })?;

    let json: Value = call_app(request).map_err(Error::from)?;
    let response = json.clone();
    let result = compiled_schema.validate(&json);

    if let Err(errors) = result {
        println!("Validation failed.");
        for error in errors {
            println!("error: {}", error);
        }
        // return Ok(CheckerLevel::Error);
    }

    Ok(response)
}

pub fn call_app<T: DeserializeOwned>(request: Value) -> Result<T> {
    ureq::post("http://localhost:8080")
        .send_json(request)
        .map_err(Error::from)?
        .into_json()
        .map_err(Error::from)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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
