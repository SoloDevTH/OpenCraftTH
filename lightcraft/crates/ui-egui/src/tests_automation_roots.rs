//! Control requests read and write files only beneath the automation roots
//! (`--automation-read-root` / `--automation-write-root`); interactive use is not confined.

use serde_json::{Value, json};

use crate::{LightcraftApp, Services};
use lightcraft_engine::automation_roots::AutomationRoots;

fn app() -> LightcraftApp {
    let write: crate::WriteFn = Box::new(|path, bytes| std::fs::write(path, bytes).map_err(|e| e.to_string()));
    LightcraftApp::new(lightcraft_engine::Session::with_demo().with_fs(), Services { write: Some(write), png: None, ..Default::default() })
}

fn call(app: &mut LightcraftApp, method: &str, params: Value) -> Value {
    let (req, _) = crate::control::ControlRequest::new(method, params);
    match crate::control::handle(app, &egui::Context::default(), &req) {
        crate::control::Outcome::Done(v) => v,
        crate::control::Outcome::Screenshot { .. } => panic!("unexpected screenshot"),
    }
}

fn files_in(dir: &std::path::Path) -> usize {
    std::fs::read_dir(dir).map_or(0, |d| d.count())
}

#[test]
fn control_file_paths_stay_inside_the_automation_roots() {
    let base = std::env::temp_dir().join(format!("lightcraft-control-roots-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (inside, outside) = (base.join("inside"), base.join("outside"));
    std::fs::create_dir_all(&inside).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    let mut a = app();
    a.automation_roots = Some(AutomationRoots::new(Some(&inside), Some(&inside)).unwrap());
    let export = |dir: &str| json!({"command": "app.export", "params": {"dir": dir, "longEdge": 32, "format": "png"}});
    for dir in [outside.to_string_lossy().into_owned(), "../outside".into()] {
        let r = call(&mut a, "engine.execute", export(&dir));
        assert_eq!(r["ok"], false, "{dir}: {r}");
        assert!(r["error"].as_str().unwrap().contains("automation path rejected"), "{r}");
        let r = call(&mut a, "engine.execute", json!({"command": "curve.exportPresets", "params": {"path": format!("{dir}/looks.lccurve")}}));
        assert_eq!(r["ok"], false, "{r}");
        assert_eq!(call(&mut a, "ui.render", json!({"path": format!("{dir}/render.png")}))["ok"], false);
        assert_eq!(call(&mut a, "ui.screenshot", json!({"path": format!("{dir}/shot.png")}))["ok"], false);
    }
    assert_eq!(files_in(&outside), 0, "nothing is written outside the root");

    let r = call(&mut a, "engine.execute", export(&inside.to_string_lossy()));
    assert_eq!(r["ok"], true, "{r}");
    assert!(files_in(&inside) > 0);
    assert!(a.session.automation_roots.is_none(), "only while the request runs");

    // Interactive use is not confined.
    assert_eq!(a.run("app.export", json!({"dir": outside.to_string_lossy(), "longEdge": 32, "format": "png"})).map(|_| ()), Ok(()));
    assert!(files_in(&outside) > 0);
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn without_roots_control_requests_touch_no_files() {
    let base = std::env::temp_dir().join(format!("lightcraft-control-noroots-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut a = app();
    a.automation_roots = Some(AutomationRoots::default());
    let r = call(&mut a, "engine.execute", json!({"command": "app.export", "params": {"dir": base.to_string_lossy(), "longEdge": 32}}));
    assert!(r["error"].as_str().unwrap().contains("not granted"), "{r}");
    assert_eq!(files_in(&base), 0);
    // Path-free commands still run.
    assert_eq!(call(&mut a, "engine.execute", json!({"command": "develop.set", "params": {"control": "light.exposure", "value": 0.5}}))["ok"], true);
    let _ = std::fs::remove_dir_all(&base);
}
