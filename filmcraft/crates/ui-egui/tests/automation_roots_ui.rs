//! Control requests read and write files only beneath the automation roots
//! (`--automation-read-root` / `--automation-write-root`); interactive use is not confined.

use std::sync::mpsc::{Sender, channel};

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_engine::automation_roots::AutomationRoots;
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::control::ControlRequest;
use serde_json::{Value, json};

struct Driver {
    harness: Harness<'static, FilmcraftApp>,
    tx: Sender<ControlRequest>,
}

impl Driver {
    fn new(roots: AutomationRoots) -> Self {
        let mut session = Session::default();
        session.execute("file.openDemoProject", json!({})).expect("demo project");
        let (tx, rx) = channel();
        let mut app = FilmcraftApp::new(session).with_control(rx);
        app.automation_roots = Some(roots);
        let harness = Harness::builder().with_size(egui::vec2(1600.0, 980.0)).with_max_steps(10_000).build_eframe(move |_cc| app);
        let mut d = Driver { harness, tx };
        d.frames(4);
        d
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let ctx = self.harness.ctx.clone();
            let mut raw = std::mem::take(self.harness.input_mut());
            eframe::App::raw_input_hook(self.harness.state_mut(), &ctx, &mut raw);
            *self.harness.input_mut() = raw;
            self.harness.step();
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let (req, reply) = ControlRequest::new(method, params.clone());
        self.tx.send(req).unwrap();
        for _ in 0..600 {
            self.frames(1);
            if let Ok(v) = reply.try_recv() {
                return v;
            }
        }
        panic!("no reply to {method} {params}");
    }
}

#[test]
fn control_file_paths_stay_inside_the_automation_roots() {
    let base = std::env::temp_dir().join(format!("filmcraft-control-roots-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let inside = base.join("inside");
    std::fs::create_dir_all(&inside).unwrap();
    let mut d = Driver::new(AutomationRoots::new(Some(&inside), Some(&inside)).unwrap());
    let outside = base.join("outside.fcproj");
    for path in [outside.to_string_lossy().into_owned(), "../outside.fcproj".into()] {
        let r = d.call("engine.execute", json!({"command": "file.saveAs", "params": {"path": path}}));
        assert_eq!(r["ok"], false, "{path}: {r}");
        assert!(r["error"].as_str().unwrap().contains("automation path rejected"), "{r}");
        let r = d.call("ui.screenshot", json!({"path": path}));
        assert_eq!(r["ok"], false, "{path}: {r}");
    }
    assert!(!outside.exists(), "nothing is written outside the root");
    assert_eq!(std::fs::read_dir(&base).unwrap().count(), 1, "only the root itself");

    let r = d.call("engine.execute", json!({"command": "file.saveAs", "params": {"path": "inside.fcproj"}}));
    assert_eq!(r["ok"], true, "{r}");
    assert!(inside.join("inside.fcproj").is_file());
    let r = d.call("engine.execute", json!({"command": "file.open", "params": {"path": "inside.fcproj"}}));
    assert_eq!(r["ok"], true, "{r}");

    // Interactive use (the session outside a control request) is not confined.
    assert!(d.harness.state().session.automation_roots.is_none());
    d.harness.state_mut().session.execute("file.saveAs", json!({"path": outside.to_string_lossy()})).unwrap();
    assert!(outside.is_file());
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn without_roots_control_requests_touch_no_files() {
    let base = std::env::temp_dir().join(format!("filmcraft-control-noroots-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let mut d = Driver::new(AutomationRoots::default());
    let target = base.join("x.fcproj");
    let r = d.call("engine.execute", json!({"command": "file.saveAs", "params": {"path": target.to_string_lossy()}}));
    assert_eq!(r["ok"], false, "{r}");
    assert!(r["error"].as_str().unwrap().contains("not granted"), "{r}");
    assert!(!target.exists());
    // Path-free commands still run.
    assert_eq!(d.call("engine.execute", json!({"command": "sequence.inspect", "params": {}}))["ok"], true);
    let _ = std::fs::remove_dir_all(&base);
}
