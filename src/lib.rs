use serde_gen::{Ty, TyBuilder};
use worker::*;

use include_dir::{include_dir, Dir};
static PROJECT_DIR: Dir<'_> = include_dir!("frontend/build");

mod utils;

fn log_request(req: &Request) {
    console_log!(
        "{} - [{}], located at: {:?}, within: {}",
        Date::now().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );
}

async fn handle_schema(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let mut headers = Headers::new();
    headers.set("content-type", "text/plain")?;
    headers.set("Access-Control-Allow-Origin", "*")?;

    let resp = match handle_schema_inner(req, ctx).await {
        Ok(res) => Ok(res),
        Err(_e) => {
            // CF workers cannot handle Err(_) case gracefully,
            Response::ok(format!("// failed to handle request: {:?}", _e))
        }
    };

    resp.map(|r| r.with_headers(headers))
}

async fn handle_schema_inner(mut req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;

    let mut root_name = std::borrow::Cow::Borrowed("Root");
    let mut tests = false;
    let mut ndjson = false;

    for (k, v) in url.query_pairs() {
        if k == "root" {
            root_name = v.to_owned();
        }
        if k == "tests" {
            tests = v == "true" || v == "";
        }
        if k == "ndjson" {
            ndjson = v == "true" || v == "";
        }
    }

    let data = req.text().await?;
    let mut ty: Ty = Ty::Unit;

    if !ndjson {
        ty = ty + serde_json::from_str(&data)?;
    } else {
        for line in data.lines() {
            ty = ty + serde_json::from_str(&line)?;
        }
    }

    let mut builder = TyBuilder::new();
    let mut out = builder.build(root_name.as_ref(), ty);

    if tests {
        use std::fmt::Write;

        let test_runner = serde_gen::TyBuilder::build_test_runner("test_runner")
            .split("\n")
            .map(|s| format!("    {}\n", s))
            .collect::<String>();

        write!(
            &mut out,
            r#"
#[cfg(test)]
mod tests {{
    use super::{0:};
{1:}
"#,
            root_name.as_ref(),
            test_runner
        )
        .ok();

        if !ndjson {
            gen_testcase_str(&mut out, "testcase", root_name.as_ref(), &data)?;
        } else {
            for (i, line) in data.lines().enumerate() {
                let tc_name = format!("testcase_{}", i);
                gen_testcase_str(&mut out, &tc_name, root_name.as_ref(), line)?;
            }
        }

        write!(&mut out, r#"}}"#,).ok();
    }

    Response::ok(out)
}

fn gen_testcase_str(out: &mut String, tc_name: &str, root_name: &str, data: &str) -> Result<()> {
    let parsed: serde_json::Value = serde_json::from_str(&data)?;
    let input_str = serde_json::to_string_pretty(&parsed)?;
    use std::fmt::Write;

    write!(
        out,
        r#"
    #[test]
    fn {4:}() {{
        const INPUT: &'static str = {2:}{1:}{3:};
        test_runner::< {0:} >(INPUT);
    }}
"#,
        root_name, input_str, "r#\"", "\"#", tc_name
    )
    .ok();

    Ok(())
}

async fn get_kv(path: &str, _ctx: &RouteContext<()>) -> Option<&'static str> {
    // trim leading slash
    let path = if path.len() > 0 && path.as_bytes()[0] == b'/' {
        &path[1..]
    } else {
        path
    };
    console_log!("query: {}", path);
    match PROJECT_DIR.get_file(path) {
        Some(file) => file.contents_utf8(),
        _ => None,
    }
}

async fn handle_static(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();

    match get_kv(path, &ctx).await {
        Some(file) => {
            let guess = mime_guess::from_path(path);

            let mut headers = Headers::new();
            let content_type = if let Some(mime) = guess.first() {
                mime.essence_str().to_string()
            } else {
                "text/plain".to_owned()
            };
            headers.set("content-type", &content_type)?;

            Response::ok(file).map(|r| r.with_headers(headers))
        }
        _ => Response::error("Not Found", 404),
    }
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    log_request(&req);

    utils::set_panic_hook();

    let router = Router::new();

    router
        .get_async("/", |_req, ctx| async move {
            match get_kv("index.html", &ctx).await {
                Some(s) => Response::from_html(&s),
                _ => Response::error("Not Found", 404),
            }
        })
        .post_async("/", handle_schema)
        .post_async("/schema", handle_schema)
        .get("/worker-version", |_, ctx| {
            let version = ctx.var("WORKERS_RS_VERSION")?.to_string();
            Response::ok(version)
        })
        .get("/debug_manifest", |_, ctx| {
            let version = ctx.var("__STATIC_CONTENT_MANIFEST")?.to_string();
            Response::ok(version)
        })
        .get_async("/static/*filepath", handle_static)
        .get_async("/ace-builds/*filepath", handle_static)
        .run(req, env)
        .await
}

#[allow(unused)]
#[durable_object]
pub struct GenSchema {
    data: String,
    env: Env, // access `Env` across requests, use inside `fetch`
}

#[durable_object]
impl DurableObject for GenSchema {
    fn new(state: State, env: Env) -> Self {
        Self {
            data: String::new(),
            env,
        }
    }

    async fn fetch(&mut self, _req: Request) -> Result<Response> {
        // do some work when a worker makes a request to this DO
        Response::ok(&format!("data: {}.", self.data))
    }
}
