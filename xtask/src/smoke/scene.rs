//! Six distinct original scene journeys. JavaScript below executes only in the browser.
use super::{
    Server,
    browser::{Bidi, Browser, Client, Launch},
    http, png, require, wait,
};
use crate::tooling::{self, Result, fail};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub const SCENARIOS: &[(&str, &str, &str)] = &[
    ("scene-chrome", "verify_browser.py", "YT-075"),
    ("scene-firefox", "verify_firefox.py", "YT-075"),
    ("scene-lifecycle-firefox", "verify_lifecycle.py", "YT-076"),
    ("scene-picking-firefox", "verify_picking.py", "YT-078"),
    ("scene-candidate-firefox", "verify_candidate.py", "YT-079"),
    ("scene-support-firefox", "verify_support.py", "YT-080"),
];
pub struct Scene<'a> {
    pub root: &'a Path,
    pub out: &'a Path,
    pub assets: PathBuf,
    pub base: String,
    pub manifest: Value,
    pub report: Value,
    browser: Option<Browser>,
    server: Option<Server>,
    temporary: Option<tempfile::TempDir>,
    deadline: Instant,
    firefox: bool,
}
pub fn run(root: &Path, out: &Path, suite: &str) -> Result<()> {
    let (_, _, task) = SCENARIOS
        .iter()
        .find(|s| s.0 == suite)
        .ok_or_else(|| fail("scene suite"))?;
    let mut s = Scene {
        root,
        out,
        assets: out.join("unused"),
        base: String::new(),
        manifest: Value::Null,
        report: json!({"task_id":task,"status":"not_run","checks":[],"actions":[],"snapshots":[],"pixels":[],"faults":[],"owned":{},"cleanup":{},"profile_changes":[],"pixel_tolerance_per_channel":25,"counters_policy":"Application counters/explicit buffer bytes, not measured VRAM","input_policy":"WebDriver mouse/keyboard; simulated touch/cancel/scroll/DPR; no physical mobile, OS IME or spoken assistive technology proof"}),
        browser: None,
        server: None,
        temporary: None,
        deadline: Instant::now()
            + Duration::from_secs(if suite.ends_with("support-firefox") {
                90
            } else if suite.contains("picking") || suite.contains("candidate") {
                180
            } else {
                600
            }),
        firefox: suite != "scene-chrome",
    };
    let result = (|| {
        s.start(suite)?;
        match suite {
            "scene-chrome" => chrome(&mut s),
            "scene-firefox" => baseline(&mut s),
            "scene-lifecycle-firefox" => super::scene_lifecycle::run(&mut s),
            "scene-picking-firefox" => super::scene_picking::run(&mut s),
            "scene-candidate-firefox" => super::scene_candidate::run(&mut s),
            "scene-support-firefox" => support(&mut s),
            _ => unreachable!(),
        }
    })();
    s.finish(result)
}
impl Scene<'_> {
    fn finish(&mut self, result: Result<()>) -> Result<()> {
        if let Err(e) = &result {
            if self.report["status"] != "gpu_unavailable" {
                self.report["status"] = json!(if self.firefox {
                    "failed"
                } else {
                    "blocked_or_failed"
                });
            }
            self.report["error"] = json!(e.message);
        }
        let cleanup = self.cleanup();
        if cleanup.is_err() {
            self.report["status"] = json!("blocked_cleanup");
        }
        if let Some(hashes) = self.report.get("input_hashes") {
            self.report["inputs_unchanged"] = json!(super::unchanged(self.root, hashes));
            if self.report["inputs_unchanged"] != true {
                self.report["status"] = json!("failed");
                self.report["error"] = json!("scene inputs changed during run");
            }
        }
        for name in ["server.log", "geckodriver.log", "chrome.log"] {
            if let Ok(bytes) = fs::read(self.out.join(name)) {
                self.report[format!("{name}_sha256")] = json!(tooling::sha(&bytes));
            }
        }
        let receipt = fs::write(
            self.out.join("result.json"),
            tooling::json_bytes(&self.report),
        )
        .map_err(|_| tooling::Error {
            code: 125,
            message: "scene report write failed".into(),
        });
        if receipt.is_err() {
            self.report["status"] = json!("blocked_report");
        }
        println!(
            "{}",
            json!({"status":self.report["status"],"outcome":self.report["outcome"],"cleanup":self.report["cleanup"],"inputs_unchanged":self.report["inputs_unchanged"]})
        );
        cleanup?;
        receipt?;
        result?;
        require(
            self.report["inputs_unchanged"] == true,
            "scene inputs changed during run",
        )
    }
}
pub(crate) fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let dst = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &dst)?;
        } else {
            require(entry.file_type()?.is_file(), "scene asset must be regular")?;
            fs::copy(entry.path(), dst)?;
        }
    }
    Ok(())
}
impl<'a> Scene<'a> {
    fn start(&mut self, suite: &str) -> Result<()> {
        let support = suite == "scene-support-firefox";
        let selected = !support && self.firefox;
        let headful = std::env::var("YT075_FIREFOX_HEADFUL_X11").as_deref() == Ok("1");
        if self.firefox && suite != "scene-firefox" {
            require(
                headful,
                "Use YT075_FIREFOX_HEADFUL_X11=1 for this selected X11 scene suite",
            )?;
        }
        let assets = self.root.join("target/webgpu-scene-island-dist");
        self.manifest = serde_json::from_slice(&fs::read(assets.join("manifest.json"))?)
            .map_err(|_| fail("scene manifest JSON"))?;
        require(
            self.manifest["rust_build_sources"] == crate::build::scene_build_sources(self.root)?,
            "Scene Rust build-source manifest missing/stale; cargo xtask build scene-island first",
        )?;
        if suite.contains("candidate") || support {
            validate_candidate(self.root, &self.manifest)?;
        }
        let mut hashes = super::source_hashes(
            self.root,
            &self.root.join("examples/webgpu-scene-island/src"),
        )?;
        for name in [
            "Cargo.toml",
            "Cargo.lock",
            "scene-loader.js",
            "counter-loader.js",
            "comparison-loader.js",
            "build.py",
            "verify_browser.py",
            "verify_firefox.py",
            "verify_lifecycle.py",
            "verify_picking.py",
            "verify_candidate.py",
            "verify_support.py",
        ] {
            let key = format!("examples/webgpu-scene-island/{name}");
            hashes[&key] = json!(tooling::sha(&tooling::read(self.root, &key)?));
        }
        for (k, v) in super::source_hashes(self.root, &assets)?
            .as_object()
            .unwrap()
        {
            hashes[k] = v.clone();
        }
        for (k, v) in super::rust_sources(self.root)?.as_object().unwrap() {
            hashes[k] = v.clone();
        }
        if let Some(sources) = self.manifest["candidate_sources"].as_object() {
            for k in sources.keys() {
                hashes[k] = json!(tooling::sha(&tooling::read(self.root, k)?));
            }
        }
        let binary = self
            .root
            .join("examples/webgpu-scene-island/target/debug")
            .join(format!(
                "yetlet-webgpu-scene-island{}",
                std::env::consts::EXE_SUFFIX
            ));
        for p in [
            &binary,
            &self.root.join("Cargo.lock"),
            &self.root.join("Cargo.toml"),
        ] {
            hashes[p.strip_prefix(self.root).unwrap().to_str().unwrap()] =
                json!(tooling::sha(&fs::read(p)?));
        }
        self.report["head"] = json!(super::head(self.root)?);
        self.report["input_hashes"] = hashes;
        self.report["manifest"] = self.manifest.clone();
        fs::write(
            self.out.join("input-hashes.json"),
            tooling::json_bytes(&self.report["input_hashes"]),
        )?;
        let temp = tempfile::Builder::new()
            .prefix("assets-")
            .tempdir_in(self.out)?;
        self.assets = temp.path().join("assets-root");
        self.temporary = Some(temp);
        copy_tree(&assets, &self.assets)?;
        self.server = Some(Server::start_binary(
            self.root,
            self.out,
            &binary,
            "YETLET_SCENE_LISTEN",
            &[(
                "YETLET_SCENE_ASSET_ROOT",
                self.assets.to_string_lossy().into_owned(),
            )],
            "/healthz",
        )?);
        let server = self.server.as_ref().unwrap();
        self.base = server.base.clone();
        self.report["owned"]["server_port"] = json!(server.port);
        self.report["owned"]["server_pid"] = json!(server.process.id());
        let response = http::get(&self.base, "/scene?q=done&page=1")?;
        require(
            response.status == 200
                && response.text()?.contains("40 matches; page 2 of 4")
                && response.text()?.contains("<html lang=\"en\">")
                && !response
                    .header("content-security-policy")?
                    .contains("unsafe-inline"),
            "scene original HTTP heading/search/pagination/English/CSP",
        )?;
        self.report["ssr"] = json!({"status":200,"http":200,"english":true,"strict_csp":true,"search_pagination":true,"search_and_pagination":true});
        let mut launch = Launch::default();
        if self.firefox {
            let home = std::env::var_os("HOME").ok_or_else(|| fail("Firefox HOME unavailable"))?;
            let common = PathBuf::from(&home).join("snap/firefox/common");
            // Keep Snap-readable profiles when the installed Firefox uses Snap.
            if common.is_dir() {
                launch.profile_parent = Some(common);
            }
            launch.firefox_options = Some(firefox_options(headful, selected));
            launch.bidi = suite.contains("picking") || suite.contains("candidate");
            launch.env = vec![
                (
                    "DISPLAY".into(),
                    Some(std::env::var_os("DISPLAY").unwrap_or(":0".into())),
                ),
                (
                    "XAUTHORITY".into(),
                    Some(
                        std::env::var_os("XAUTHORITY")
                            .unwrap_or(PathBuf::from(home).join(".Xauthority").into_os_string()),
                    ),
                ),
            ];
            self.report["profile_changes"] = if selected {
                json!(["dom.webgpu.enabled=true (temporary only)"])
            } else {
                json!([])
            };
            self.report["requested_preferences"] = if selected {
                json!({"dom.webgpu.enabled":true})
            } else {
                json!({})
            };
            self.report["mode"] = json!(if headful {
                "Firefox X11"
            } else {
                "Firefox headless"
            });
        } else {
            let (config, name) = chrome_launch()?;
            launch = config;
            self.report["gpu_profile"] = json!(name);
        }
        self.browser = Some(Browser::start_config(
            self.root,
            self.out,
            self.firefox,
            launch,
        )?);
        let b = self.browser.as_mut().unwrap();
        self.report["owned"][if self.firefox {
            "driver_port"
        } else {
            "chrome_port"
        }] = json!(b.port());
        self.report["owned"][if self.firefox {
            "geckodriver_pid"
        } else {
            "browser_pid"
        }] = json!(b.pid());
        self.report["browser"] = json!(b.version);
        if let Client::Firefox(f) = b.client()? {
            f.deadline = Some(self.deadline);
            self.report["capabilities"] = f.capabilities.clone();
            if suite.contains("candidate") || suite.contains("picking") || support {
                require(
                    f.capabilities["browserVersion"]
                        .as_str()
                        .is_some_and(|v| v.starts_with("158.")),
                    "Selected Firefox 158 required",
                )?;
            }
            f.command("/timeouts","POST",Some(&json!({"script":if support {15000}else{25000},"pageLoad":if support{20000}else{30000}})))?;
        }
        self.report["status"] = json!("running");
        Ok(())
    }
    pub fn client(&mut self) -> Result<&mut Client> {
        self.tick()?;
        self.browser
            .as_mut()
            .ok_or_else(|| fail("scene browser missing"))?
            .client()
    }
    pub fn tick(&self) -> Result<()> {
        tooling::check_cancel()?;
        if Instant::now() >= self.deadline {
            return Err(tooling::Error {
                code: 124,
                message: "scene focused test deadline".into(),
            });
        }
        Ok(())
    }
    pub fn eval(&mut self, js: &str) -> Result<Value> {
        self.client()?.evaluate(js)
    }
    pub fn exec(&mut self, js: &str) -> Result<Value> {
        self.eval(&format!("(()=>{{{js}}})()"))
    }
    pub fn check(&mut self, js: &str) -> Result<()> {
        require(self.eval(js)? == true, &format!("scene assertion: {js}"))
    }
    pub fn assert_script(&mut self, js: &str) -> Result<()> {
        require(self.exec(js)? == true, &format!("scene assertion: {js}"))
    }
    pub fn async_js(&mut self, js: &str) -> Result<Value> {
        match self.client()? {
            Client::Firefox(f) => f.async_script(js),
            Client::Chrome(c) => c.evaluate(js),
        }
    }
    pub fn until(&mut self, js: &str, label: &str, seconds: u64) -> Result<()> {
        wait(seconds, label, || {
            self.tick()?;
            Ok(self.eval(js)?.as_bool().filter(|v| *v).map(|_| ()))
        })
    }
    pub fn frames(&mut self) -> Result<()> {
        if self.firefox {
            self.async_js("const done=arguments[arguments.length-1];requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>done(null)))));")?;
        } else {
            self.eval(
                "new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))",
            )?;
        }
        Ok(())
    }
    pub fn fresh(&mut self, label: &str) -> Result<()> {
        self.navigate(&format!("/scene?yt085={label}"), true)?;
        self.exec("document.getElementById('scene-canvas').scrollIntoView({block:'center'})")?;
        self.frames()
    }
    pub fn navigate(&mut self, path: &str, enhanced: bool) -> Result<()> {
        let url = format!("{}{path}", self.base);
        self.client()?.navigate(&url)?;
        self.until(
            &format!(
                "document.readyState==='complete' && location.href==={}",
                json!(url)
            ),
            "fresh scene document",
            20,
        )?;
        if enhanced {
            self.until("!!window.__yt075Scene && document.querySelector('#counter-island')?.dataset.yetletActivated==='true'","scene and sibling loaders",22)?;
        }
        self.snap("fresh navigation")?;
        Ok(())
    }
    pub fn snap(&mut self, label: &str) -> Result<Value> {
        let data=self.eval("({url:location.href,ready:document.readyState,scene:window.__yt075Scene?.snapshot()??null,state:window.__yt075Scene?.snapshot()??null,text:document.querySelector('#scene-status')?.textContent,status:document.querySelector('#scene-status')?.textContent,counter:document.querySelector('#counter-value')?.textContent,records:document.querySelectorAll('#records li').length,records_text:document.querySelector('#records')?.textContent,adapter:document.querySelector('#scene-island')?.dataset.adapter,canvas:{width:document.querySelector('#scene-canvas')?.width,height:document.querySelector('#scene-canvas')?.height}})")?;
        self.report["snapshots"]
            .as_array_mut()
            .unwrap()
            .push(json!({"label":label,"value":data}));
        Ok(data["scene"].clone())
    }
    pub fn click_id(&mut self, id: &str) -> Result<()> {
        self.exec(&format!("document.getElementById({}).click()", json!(id)))?;
        Ok(())
    }
    pub fn state(&mut self, name: &str) -> Result<()> {
        self.until(
            &format!("window.__yt075Scene.snapshot().state==={}", json!(name)),
            name,
            40,
        )?;
        self.frames()?;
        require(self.snap(name)?["state"] == name, "settled scene state")
    }
    pub fn activate(&mut self) -> Result<()> {
        self.click_id("activate-scene")?;
        self.exec("document.getElementById('scene-canvas').scrollIntoView({block:'center'})")?;
        self.state("ready")
    }
    pub fn record(&mut self, name: &str) {
        self.report["checks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":name,"status":"passed"}));
    }
    pub fn adapter(&mut self) -> Result<Value> {
        let a = if self.firefox {
            self.async_js("const done=arguments[arguments.length-1];(async()=>{const a=await navigator.gpu?.requestAdapter();done(a?{available:true,fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null,info:a.info?{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}:null}:{available:!!navigator.gpu,adapter:null})})().catch(e=>done({error:String(e)}));")?
        } else {
            self.eval("(async()=>{const a=await navigator.gpu?.requestAdapter();return a?{available:true,fallback:a.isFallbackAdapter??a.info?.isFallbackAdapter??null,info:{vendor:a.info.vendor,architecture:a.info.architecture,device:a.info.device,description:a.info.description}}:{available:!!navigator.gpu,adapter:null}})()")?
        };
        self.report["adapter"] = a.clone();
        self.report["navigator_adapter"] = a.clone();
        Ok(a)
    }
    pub fn real_adapter(&mut self) -> Result<()> {
        let a = self.adapter()?;
        hardware_adapter(&a)
    }
    pub fn pixels(&mut self, label: &str, colors: Option<&[usize]>, exact: bool) -> Result<Value> {
        self.exec("window.__yt075Scene.draw()")?;
        if self.firefox {
            self.async_js("const done=arguments[arguments.length-1];window.__yt075Scene.completion().then(()=>requestAnimationFrame(()=>requestAnimationFrame(()=>done(null))));")?;
        } else {
            self.frames()?;
        }
        let css=self.eval("(()=>{const b=document.getElementById('scene-canvas').getBoundingClientRect();return {width:b.width,height:b.height,x:b.x+scrollX,y:b.y+scrollY,scale:1}})()")?;
        let encoded = match self.client()? {
            Client::Firefox(f) => {
                let id = f.element("#scene-canvas")?;
                f.command(&format!("/element/{id}/screenshot"), "GET", None)?
            }
            Client::Chrome(c) => c.call(
                "Page.captureScreenshot",
                json!({"clip":css,"captureBeyondViewport":true}),
            )?["data"]
                .clone(),
        };
        let image = STANDARD
            .decode(
                encoded
                    .as_str()
                    .ok_or_else(|| fail("scene screenshot encoding"))?,
            )
            .map_err(|_| fail("scene PNG base64"))?;
        fs::write(self.out.join(format!("{label}.png")), &image)?;
        let p = png::decode(&image)?;
        require(
            (p.width as f64 - css["width"].as_f64().unwrap()).abs() <= 1.
                && (p.height as f64 - css["height"].as_f64().unwrap()).abs() <= 1.,
            "scene screenshot CSS shape",
        )?;
        let proof = pixel_proof(&p, colors, exact)?;
        let mut proof = proof;
        proof["label"] = json!(label);
        proof["sha256"] = json!(tooling::sha(&image));
        self.report["pixels"]
            .as_array_mut()
            .unwrap()
            .push(proof.clone());
        require(
            proof["mismatches"].as_array().unwrap().is_empty()
                && proof["interior_black_samples"] == json!([[0, 0, 0], [0, 0, 0]]),
            "Real canvas 120 centers / black interior failed",
        )?;
        Ok(proof)
    }
    pub fn portrait(&mut self) -> Result<Value> {
        let url = self.eval("location.href")?;
        let ws = self.client()?.firefox()?.capabilities["webSocketUrl"]
            .as_str()
            .ok_or_else(|| fail("BiDi URL missing"))?
            .to_owned();
        let mut b = Bidi::connect(&ws)?;
        let tree = b.call("browsingContext.getTree", json!({"maxDepth":0}))?;
        let matches: Vec<_> = tree["contexts"]
            .as_array()
            .ok_or_else(|| fail("BiDi contexts"))?
            .iter()
            .filter(|c| c["url"] == url)
            .collect();
        require(matches.len() == 1, "one owned BiDi page")?;
        let context = matches[0]["context"].clone();
        b.call(
            "browsingContext.setViewport",
            json!({"context":context,"viewport":{"width":390,"height":700}}),
        )?;
        Ok(
            json!({"context":context,"width":390,"height":700,"kind":"BiDi viewport emulation; not physical mobile"}),
        )
    }
    fn cleanup(&mut self) -> Result<()> {
        // Attempt every owned teardown even if an earlier teardown failed.
        if let Some(Client::Firefox(f)) = self.browser.as_mut().and_then(|b| b.client.as_mut()) {
            f.deadline = None;
        }
        let browser = self.browser.as_mut().map_or(Ok(()), Browser::stop);
        if let Some(b) = &self.browser {
            self.report["cleanup"]["browser_exit"] = json!(b.exit_code());
            if self.firefox {
                self.report["cleanup"]["geckodriver_exit"] = json!(b.exit_code());
            }
        }
        self.browser.take();
        let server = self.server.as_mut().map_or(Ok(()), |s| s.stop(false));
        if let Some(s) = &self.server
            && let Ok(bytes) = fs::read(s.log.with_extension("exit.json"))
            && let Ok(v) = serde_json::from_slice::<Value>(&bytes)
        {
            self.report["cleanup"]["server_exit"] = v["native_exit_code"].clone();
        }
        self.server.take();
        let temp = self.temporary.take().map_or(Ok(()), |t| {
            t.close().map_err(|_| tooling::Error {
                code: 125,
                message: "scene temporary directory cleanup failed".into(),
            })
        });
        self.report["cleanup"]["session_deleted"] = json!(browser.is_ok());
        self.report["cleanup"]["tmp_profile_removed"] = json!(browser.is_ok() && temp.is_ok());
        self.report["cleanup"]["profile_removed"] = json!(browser.is_ok() && temp.is_ok());
        self.report["cleanup"]["processes_reaped"] = json!(browser.is_ok() && server.is_ok());
        self.report["cleanup"]["browser_exited"] = json!(browser.is_ok());
        self.report["cleanup"]["ports_closed"] =
            json!({"browser":browser.is_ok(),"server":server.is_ok()});
        let mut errors = vec![];
        for (name, result) in [
            ("browser", &browser),
            ("server", &server),
            ("profile", &temp),
        ] {
            if let Err(e) = result {
                errors.push(format!("{name}: {}", e.message));
                self.report["cleanup"][format!("{name}_error")] = json!(e.message);
            }
        }
        self.report["cleanup"]["errors"] = json!(errors);
        browser?;
        server?;
        temp
    }
}
pub fn firefox_options(headful: bool, experimental: bool) -> Value {
    json!({"args":if headful{Vec::<&str>::new()}else{vec!["-headless"]},"prefs":if experimental{json!({"dom.webgpu.enabled":true})}else{json!({})}})
}
fn hardware_adapter(a: &Value) -> Result<()> {
    require(
        a["available"] == true
            && a.get("adapter").is_none()
            && a["fallback"] != true
            && !["swiftshader", "llvmpipe", "lavapipe", "software"]
                .iter()
                .any(|bad| a.to_string().to_lowercase().contains(bad)),
        "Actual non-fallback hardware adapter required",
    )
}
/// The current loader adds global retirement diagnostics to the old full snapshot.
/// Compare all current-owner fields, and assert those global transitions separately.
pub fn same_current_owner(before: &Value, after: &Value) -> bool {
    fn owner(v: &Value) -> Value {
        let mut v = v.clone();
        if let Some(map) = v.as_object_mut() {
            map.remove("pendingInitializations");
            map.remove("lateSettled");
            if let Some(retired) = map.get_mut("lastRetired").and_then(Value::as_object_mut) {
                retired.remove("resourcesReleased");
            }
        }
        v
    }
    owner(before) == owner(after)
}
pub fn validate_candidate(root: &Path, manifest: &Value) -> Result<()> {
    let sources = manifest["candidate_sources"]
        .as_object()
        .ok_or_else(|| fail("Source-bound extracted candidate manifest required"))?;
    require(
        sources.contains_key("crates/yetlet-browser/src/wgpu/browser.rs"),
        "candidate browser source missing",
    )?;
    for (p, h) in sources {
        require(
            h.as_str() == Some(tooling::sha(&tooling::read(root, p)?).as_str()),
            "Stale candidate source/build",
        )?;
    }
    Ok(())
}
/// Fixed grid, fixed tolerance. Only an entirely white first row shifts capture origin.
pub fn pixel_proof(p: &png::Pixels, colors: Option<&[usize]>, exact: bool) -> Result<Value> {
    require(p.width >= 12 && p.height >= 10, "scene pixel dimensions")?;
    if let Some(colors) = colors {
        require(
            colors.len() == 120 && colors.iter().all(|v| *v < 3),
            "scene color mapping range",
        )?;
    }
    let top = if !exact && (0..p.width).all(|x| p.rgb(x, 0).is_ok_and(|v| v == [255, 255, 255])) {
        1
    } else {
        0
    };
    let mut mismatches = vec![];
    for i in 0..120 {
        let x = ((i % 12) as f64 + 0.5) * p.width as f64 / 12.;
        let y = ((i / 12) as f64 + 0.5) * p.height as f64 / 10.;
        let actual = p.rgb(x as usize, y as usize + top)?;
        let expected = [[255u8, 0, 0], [0, 255, 0], [0, 0, 255]][colors.map_or(i % 3, |c| c[i])];
        if actual
            .iter()
            .zip(expected)
            .any(|(a, b)| a.abs_diff(b) > if exact { 0 } else { 25 })
        {
            mismatches.push(json!({"id":i+1,"capture":[x as usize,y as usize+top],"actual":actual,"expected":expected}));
        }
    }
    let black = p.rgb(2, top + 2)?;
    let second_black = p.rgb(p.width / 12, top + p.height / 20)?;
    if exact {
        require(p.rgb(0, 0)? == [0, 0, 0], "Chrome black margin")?;
    }
    Ok(
        json!({"width":p.width,"height":p.height,"capture_canvas_origin":[0,top],"capture_top_edge":p.rgb(0,0)?,"verified":120-mismatches.len(),"centers_verified":120-mismatches.len(),"mismatches":mismatches,"interior_black":black,"interior_black_samples":[black,second_black]}),
    )
}
fn chrome_launch() -> Result<(Launch, String)> {
    let names = [
        "YT075_NVIDIA_ICD_DIAGNOSTIC",
        "YT075_PRIME_OFFLOAD_DIAGNOSTIC",
        "YT075_INTEL_GEN12_DIAGNOSTIC",
        "YT075_NESTED_WESTON_DIAGNOSTIC",
        "YT075_ANGLE_VULKAN_SHARING_DIAGNOSTIC",
    ];
    let enabled: Vec<_> = names
        .iter()
        .enumerate()
        .filter(|(_, n)| std::env::var(n).as_deref() == Ok("1"))
        .map(|(i, _)| i)
        .collect();
    require(
        enabled.len() <= 1,
        "Select only one YT-075 GPU diagnostic profile",
    )?;
    let mut c = Launch {
        chrome_args: Some(vec![
            "--headless=new".into(),
            "--enable-gpu".into(),
            "--ozone-platform=x11".into(),
        ]),
        ..Launch::default()
    };
    c.env.push(("DISPLAY".into(), Some(":0".into())));
    let home = std::env::var_os("HOME").ok_or_else(|| fail("Chrome HOME"))?;
    c.env.push((
        "XAUTHORITY".into(),
        Some(PathBuf::from(home).join(".Xauthority").into_os_string()),
    ));
    let mut profile = "x11-default";
    if let Some(i) = enabled.first() {
        let args = c.chrome_args.as_mut().unwrap();
        match i {
            0 => {
                require(
                    Path::new("/usr/share/vulkan/icd.d/nvidia_icd.json").is_file(),
                    "NVIDIA Vulkan ICD unavailable",
                )?;
                c.env.push((
                    "VK_DRIVER_FILES".into(),
                    Some("/usr/share/vulkan/icd.d/nvidia_icd.json".into()),
                ));
                profile = "nvidia-icd-only";
            }
            1 | 4 => {
                c.env.extend([
                    ("VK_DRIVER_FILES".into(), None),
                    ("VK_ICD_FILENAMES".into(), None),
                ]);
                profile = if *i == 1 {
                    "prime-offload-default-icds"
                } else {
                    "angle-vulkan-sharing-x11-nvidia"
                };
            }
            2 => {
                for n in [
                    "VK_DRIVER_FILES",
                    "VK_ICD_FILENAMES",
                    "__NV_PRIME_RENDER_OFFLOAD",
                    "__VK_LAYER_NV_optimus",
                    "__GLX_VENDOR_LIBRARY_NAME",
                ] {
                    c.env.push((n.into(), None));
                }
                c.env.extend([
                    ("DRI_PRIME".into(), Some("0".into())),
                    ("MESA_VK_DEVICE_SELECT".into(), Some("8086:a780!".into())),
                ]);
                args.push("--use-angle=gl".into());
                profile = "intel-gen12-default-icds";
            }
            3 => {
                let socket = "wayland-yt075-weston-browser";
                let runtime =
                    std::env::var_os("XDG_RUNTIME_DIR").unwrap_or("/run/user/1000".into());
                require(
                    PathBuf::from(runtime).join(socket).exists(),
                    "Owned Weston Wayland socket missing",
                )?;
                args[2] = "--ozone-platform=wayland".into();
                c.env.extend([
                    ("WAYLAND_DISPLAY".into(), Some(socket.into())),
                    ("XDG_SESSION_TYPE".into(), Some("wayland".into())),
                ]);
                profile = "nested-weston-x11-nvidia-wayland";
            }
            _ => unreachable!(),
        }
        if *i != 2 {
            if *i == 3 || *i == 4 {
                for n in [
                    "VK_DRIVER_FILES",
                    "VK_ICD_FILENAMES",
                    "DRI_PRIME",
                    "MESA_VK_DEVICE_SELECT",
                ] {
                    c.env.push((n.into(), None));
                }
            }
            c.env.extend([
                ("__NV_PRIME_RENDER_OFFLOAD".into(), Some("1".into())),
                ("__VK_LAYER_NV_optimus".into(), Some("NVIDIA_only".into())),
            ]);
            if *i != 0 {
                c.env
                    .push(("__GLX_VENDOR_LIBRARY_NAME".into(), Some("nvidia".into())));
            }
            args.push("--use-angle=vulkan".into());
            args.push(
                if *i == 4 {
                    "--enable-features=Vulkan,VulkanFromANGLE,DefaultANGLEVulkan"
                } else {
                    "--enable-features=Vulkan"
                }
                .into(),
            );
        }
    }
    Ok((c, profile.into()))
}
const HELD_DEVICE: &str = r#"window.__yt075Held={calls:0,destroyed:0,acquired:false};
const request=GPUAdapter.prototype.requestDevice;
GPUAdapter.prototype.requestDevice=async function(...args){
 const first=++window.__yt075Held.calls===1;const device=await request.apply(this,args);
 if(!first)return device;
 const destroy=device.destroy.bind(device);device.destroy=()=>{window.__yt075Held.destroyed++;destroy()};
 window.__yt075Held.acquired=true;return await new Promise(resolve=>{window.__yt075Held.release=()=>resolve(device)});
};"#;
fn held_device(s: &mut Scene<'_>) -> Result<()> {
    s.fresh("late_completion")?;
    s.exec(HELD_DEVICE)?;
    s.click_id("activate-scene")?;
    s.until("window.__yt075Held.acquired", "held real device", 35)?;
    s.click_id("activate-scene")?;
    require(
        s.snap("duplicate while pending")?["starts"] == 1,
        "single pending start",
    )?;
    s.check("window.__yt075Held.calls===1")?;
    s.click_id("dispose-scene")?;
    s.activate()?;
    let replacement = s.snap("replacement before late completion")?;
    let text = s.eval("document.getElementById('scene-status').textContent")?;
    s.exec("window.__yt075Held.release()")?;
    s.until(
        "window.__yt075Held.destroyed===1",
        "retired device destroyed",
        25,
    )?;
    let after = s.snap("replacement after late completion")?;
    require(
        same_current_owner(&replacement, &after)
            && after["pendingInitializations"] == 0
            && after["lateSettled"] == 1
            && after["lastRetired"]["resourcesReleased"] == true
            && s.eval("document.getElementById('scene-status').textContent")? == text,
        "replacement unchanged by stale completion",
    )?;
    s.click_id("counter-increment")?;
    s.check("document.getElementById('counter-value').textContent==='3'")?;
    s.click_id("dispose-scene")?;
    s.report["faults"].as_array_mut().unwrap().push(json!({"name":"held_real_device_retirement","injected":true,"stale_completion_destroyed":true,"replacement_and_sibling":"passed"}));
    Ok(())
}
fn baseline(s: &mut Scene<'_>) -> Result<()> {
    s.fresh("baseline")?;
    let initial = s.snap("before activation")?;
    require(initial["starts"] == 0, "initial scene inert")?;
    s.check("document.querySelectorAll('#records li').length===12 && document.getElementById('scene-canvas').width===600 && document.getElementById('scene-canvas').height===400 && !performance.getEntriesByType('resource').some(x=>x.name.includes('scene_bg.wasm')||x.name.endsWith('/scene.js'))")?;
    let adapter = s.adapter()?;
    s.click_id("activate-scene")?;
    s.until(
        "['ready','unavailable'].includes(window.__yt075Scene.snapshot().state)",
        "Firefox GPU initialization",
        45,
    )?;
    if s.snap("GPU activation")?["state"] != "ready" {
        s.report["status"] = json!("gpu_unavailable");
        return Err(fail("Required GPU capability unavailable"));
    }
    hardware_adapter(&adapter)?;
    let proof = s.pixels("actual-canvas", None, false)?;
    require(
        proof["width"] == 600 && proof["height"] == 400,
        "original canvas 600x400",
    )?;
    s.check("(()=>{const c=document.getElementById('scene-canvas');return !!c})()")?;
    // Original baseline's two interior black samples, excluding the crop seam.
    let p = png::decode(&fs::read(s.out.join("actual-canvas.png"))?)?;
    require(
        p.rgb(5, 1)? == [0, 0, 0] && p.rgb(45, 20)? == [0, 0, 0],
        "baseline two interior black points",
    )?;
    s.report["actual_pixels"] = proof;
    s.report["pre_capture_redraw"] = json!("draw_submitted");
    let before = s.snap("repeated activation before")?;
    s.click_id("activate-scene")?;
    require(
        s.snap("repeated activation after")?["starts"] == before["starts"],
        "no duplicate mount",
    )?;
    s.click_id("counter-increment")?;
    s.check("document.getElementById('counter-value').textContent==='3'")?;
    s.click_id("dispose-scene")?;
    require(
        s.snap("after dispose")?["state"] == "disposed",
        "explicit dispose",
    )?;
    s.activate()?;
    require(
        s.snap("after remount")?["starts"] == 2,
        "remount start count",
    )?;
    s.click_id("dispose-scene")?;
    require(
        s.snap("after second dispose")?["state"] == "disposed",
        "second disposal",
    )?;
    s.report["completed_browser_checks"] = json!([
        "120 real canvas pixel centers",
        "interior black clear",
        "single pending initialization",
        "independent sibling counter",
        "explicit resource dispose",
        "dispose and remount"
    ]);
    held_device(s)?;
    for (name, injection) in [
        (
            "absent_webgpu",
            "Object.defineProperty(navigator,'gpu',{get:()=>undefined,configurable:true});",
        ),
        (
            "null_adapter",
            "Object.defineProperty(navigator.gpu,'requestAdapter',{value:async()=>null,configurable:true});",
        ),
        (
            "device_rejection",
            "Object.defineProperty(GPUAdapter.prototype,'requestDevice',{value:async()=>{throw new Error('injected device failure')},configurable:true});",
        ),
        (
            "initialization_validation",
            "Object.defineProperty(GPUDevice.prototype,'popErrorScope',{value:async()=>new GPUValidationError({message:'injected validation error'}),configurable:true});",
        ),
    ] {
        s.fresh(name)?;
        require(
            s.snap(&format!("fresh {name}"))?["starts"] == 0,
            "fresh owner starts zero",
        )?;
        s.check("document.getElementById('counter-value').textContent==='2'")?;
        s.exec(injection)?;
        s.click_id("activate-scene")?;
        s.until(
            "window.__yt075Scene.snapshot().state==='unavailable'",
            name,
            35,
        )?;
        s.check("document.querySelectorAll('#records li').length===12 && document.getElementById('scene-status').textContent.includes('HTML list')")?;
        s.click_id("counter-increment")?;
        s.check("document.getElementById('counter-value').textContent==='3'")?;
        s.report["faults"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":name,"injected":true,"fallback_and_sibling":"passed"}));
    }
    let manifest = s.assets.join("manifest.json");
    let bytes = fs::read(&manifest)?;
    let wasm = s
        .assets
        .join("assets")
        .join(
            s.manifest["scene"]["digest"]
                .as_str()
                .ok_or_else(|| fail("scene digest"))?,
        )
        .join("scene_bg.wasm");
    let wasm_bytes = fs::read(&wasm)?;
    fs::remove_file(&wasm)?;
    let r = (|| {
        s.navigate("/scene?yt075=missing_wasm", false)?;
        s.check("!window.__yt075Scene && document.querySelectorAll('#records li').length===12")
    })();
    fs::write(&wasm, wasm_bytes)?;
    r?;
    s.report["faults"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"missing_wasm","semantic_ssr":"passed"}));
    fs::write(&manifest, b"invalid")?;
    let r = (|| {
        let r = http::get(&s.base, "/scene?record=38&q=done&page=1")?;
        require(
            r.status == 200
                && r.text()?.contains("Work record 038")
                && !r.text()?.contains("<script")
                && !r
                    .header("content-security-policy")?
                    .contains("unsafe-inline"),
            "invalid manifest semantic SSR",
        )
    })();
    fs::write(&manifest, bytes)?;
    r?;
    s.report["faults"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name":"invalid_manifest","semantic_ssr":"passed"}));
    s.report["completed_browser_checks"]
        .as_array_mut()
        .unwrap()
        .push(json!("missing Wasm and malformed manifest fail closed"));
    s.report["status"] = json!("passed_y075_browser_qualification");
    Ok(())
}
fn chrome(s: &mut Scene<'_>) -> Result<()> {
    // Actual script-disabled semantic search/pagination/detail on the owned page.
    s.client()?.chrome()?.call(
        "Emulation.setScriptExecutionDisabled",
        json!({"value":true}),
    )?;
    let url = format!("{}/scene?q=Done&page=1&record=38", s.base);
    s.client()?.navigate(&url)?;
    let html = wait(20, "no-JS semantic document", || {
        let c = s.client()?.chrome()?;
        let node = c.call("DOM.getDocument", json!({}))?["root"]["nodeId"].clone();
        let html = c.call("DOM.getOuterHTML", json!({"nodeId":node}))?["outerHTML"]
            .as_str()
            .unwrap_or("")
            .to_owned();
        Ok((html.contains("40 matches; page 2 of 4")
            && html.contains("Work record 038")
            && html.contains("method=\"get\""))
        .then_some(html))
    })?;
    s.report["no_js_html"] = json!(html);
    let data = s
        .client()?
        .chrome()?
        .call("Page.captureScreenshot", json!({}))?["data"]
        .as_str()
        .unwrap_or("")
        .to_owned();
    fs::write(
        s.out.join("no-js.png"),
        STANDARD.decode(data).map_err(|_| fail("no-JS PNG"))?,
    )?;
    s.client()?.chrome()?.call(
        "Emulation.setScriptExecutionDisabled",
        json!({"value":false}),
    )?;
    s.fresh("hardware")?;
    require(
        !s.client()?
            .chrome()?
            .requests
            .iter()
            .any(|u| u.contains("/scene.js") || u.contains("scene_bg.wasm")),
        "no GPU files before activation",
    )?;
    let adapter = s.adapter()?;
    // SystemInfo is browser-scoped; use the debugger's browser endpoint.
    let b = s.browser.as_ref().unwrap();
    let port = b.port();
    let version = http::get(&format!("http://127.0.0.1:{port}"), "/json/version")?.json()?;
    let mut browser = super::browser::Cdp::connect(
        version["webSocketDebuggerUrl"]
            .as_str()
            .ok_or_else(|| fail("browser debugger URL"))?,
    )?;
    s.report["chrome"] = version;
    s.report["owned_target"] = s
        .client()?
        .chrome()?
        .call("Target.getTargetInfo", json!({}))?["targetInfo"]["targetId"]
        .clone();
    s.report["system_gpu_observation"] =
        browser.call("SystemInfo.getInfo", json!({}))?["gpu"].clone();
    s.click_id("activate-scene")?;
    s.until(
        "['ready','unavailable'].includes(window.__yt075Scene.snapshot().state)",
        "fallible GPU initialization",
        40,
    )?;
    if s.snap("actual GPU result")?["state"] != "ready" {
        s.report["status"] = json!("gpu_unavailable");
        return Err(fail(
            "Required GPU capability unavailable; stop without GPU acceptance",
        ));
    }
    hardware_adapter(&adapter)?;
    let proof = s.pixels("actual-canvas", None, true)?;
    require(
        proof["width"] == 600 && proof["height"] == 400,
        "Chrome original canvas shape",
    )?;
    s.report["actual_pixels"] =
        json!({"centers":120,"margin":"black","screenshot":"actual-canvas.png"});
    for (name, injection) in [
        (
            "absent_webgpu",
            "Object.defineProperty(Navigator.prototype,'gpu',{get:()=>undefined,configurable:true});",
        ),
        (
            "null_adapter",
            "navigator.gpu.requestAdapter=async()=>null;",
        ),
        (
            "device_rejection",
            "GPUAdapter.prototype.requestDevice=async()=>{throw new Error('YT-075 injected device failure')};",
        ),
        (
            "initialization_validation",
            "GPUDevice.prototype.popErrorScope=async()=>new GPUValidationError('YT-075 injected initialization validation');",
        ),
    ] {
        let id = s.client()?.chrome()?.call(
            "Page.addScriptToEvaluateOnNewDocument",
            json!({"source":injection}),
        )?["identifier"]
            .clone();
        let r = (|| {
            s.fresh(name)?;
            s.click_id("activate-scene")?;
            s.until(
                "window.__yt075Scene.snapshot().state==='unavailable'",
                name,
                35,
            )?;
            s.click_id("counter-increment")?;
            s.check("document.getElementById('counter-value').textContent==='3' && document.querySelectorAll('#records li').length===12")
        })();
        s.client()?.chrome()?.call(
            "Page.removeScriptToEvaluateOnNewDocument",
            json!({"identifier":id}),
        )?;
        r?;
        s.report["faults"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":name,"injected":true,"fallback_and_sibling":"passed"}));
    }
    s.client()?
        .chrome()?
        .call("Network.setBlockedURLs", json!({"urls":["*scene_bg.wasm"]}))?;
    let r = (|| {
        s.fresh("missing-wasm")?;
        s.click_id("activate-scene")?;
        s.until(
            "window.__yt075Scene.snapshot().state==='unavailable'",
            "missing Wasm",
            35,
        )?;
        s.click_id("counter-increment")?;
        s.check("document.getElementById('counter-value').textContent==='3'")
    })();
    s.client()?
        .chrome()?
        .call("Network.setBlockedURLs", json!({"urls":[]}))?;
    r?;
    s.report["faults"].as_array_mut().unwrap().push(json!({"name":"missing_wasm_network_block","injected":true,"fallback_and_sibling":"passed"}));
    let path = s.assets.join("manifest.json");
    let saved = fs::read(&path)?;
    for (name, data) in [
        ("missing_manifest", None),
        ("invalid_manifest", Some(b"invalid".as_slice())),
    ] {
        if let Some(data) = data {
            fs::write(&path, data)?;
        } else {
            fs::remove_file(&path)?;
        }
        let r = (|| {
            s.navigate(&format!("/scene?yt085={name}"), false)?;
            s.check("!window.__yt075Scene && document.querySelectorAll('#records li').length===12 && !!document.querySelector('form[method=get]')")
        })();
        fs::write(&path, &saved)?;
        r?;
        s.report["faults"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":name,"injected":true,"semantic_ssr":"passed"}));
    }
    held_device(s)?;
    let c = s.client()?.chrome()?;
    require(
        c.api_posts.is_empty() && c.websockets.is_empty(),
        "no remote event forwarding",
    )?;
    let exceptions = c.exceptions.clone();
    let failures = c.failed_responses.clone();
    s.report["browser_exceptions"] = json!(exceptions);
    s.report["network_failures"] = json!(failures);
    s.report["status"] = json!("passed");
    Ok(())
}
fn support(s: &mut Scene<'_>) -> Result<()> {
    s.report["profile_boundary"] = json!(
        "Fresh WebDriver profile; automation defaults may apply, no WebGPU preference override"
    );
    s.client()?.firefox()?.command(
        "/window/rect",
        "POST",
        Some(&json!({"width":1100,"height":1000})),
    )?;
    s.fresh("default-profile")?;
    require(
        s.snap("initial inert candidate")?["starts"] == 0,
        "default initial inert",
    )?;
    s.check("document.querySelectorAll('#records li').length===12 && !performance.getEntriesByType('resource').some(x=>x.name.includes('scene_bg.wasm'))")?;
    s.report["capability"] =
        s.eval("({secure:window.isSecureContext,navigator_gpu:!!navigator.gpu})")?;
    s.native_click(
        "#activate-scene",
        "explicit activation with no preference override",
    )?;
    s.script(
        "#scene-canvas",
        "restore visible surface",
        "document.getElementById('scene-canvas').scrollIntoView({block:'center'})",
    )?;
    s.until(
        "['ready','unavailable'].includes(window.__yt075Scene.snapshot().state)",
        "default capability outcome",
        25,
    )?;
    let state = s.snap("actual default outcome")?;
    let outcome = state["state"].as_str().unwrap_or("unknown").to_owned();
    s.report["outcome"] = json!(outcome);
    s.check("document.querySelectorAll('#records li').length===12 && document.getElementById('counter-value').textContent==='2'")?;
    if outcome == "ready" {
        let a = s.adapter()?;
        require(a["fallback"] == false, "non-fallback default adapter")?;
        let p = s.pixels("canvas", None, false)?;
        s.report["pixels"] = p;
    } else {
        require(
            state["ownedBufferBytes"] == 0 && state["ownedSurfaces"] == 0,
            "default fallback no GPU ownership",
        )?;
        s.check("document.getElementById('scene-status').textContent.includes('HTML')")?;
        s.report["fallback"] =
            json!("Actual unavailable state; semantic records and controls remain usable");
    }
    s.native_click("#counter-increment", "independent counter")?;
    s.check("document.getElementById('counter-value').textContent==='3'")?;
    s.record("independent counter");
    s.native_click("#scene-navigate", "focus semantic keyboard alternative")?;
    s.native_keys("#scene-navigate", "select last record", "\u{e010}")?;
    s.native_keys("#scene-navigate", "commit semantic selection", "\u{e007}")?;
    s.check("window.__yt078Selection.snapshot().selectedRecord===120 && document.getElementById('scene-detail-link').getAttribute('href')==='/scene?record=120'")?;
    s.native_click("#scene-detail-link", "native detail navigation")?;
    s.check("document.getElementById('detail').textContent.includes('Work record 120')")?;
    s.record("native detail navigation and semantic selection");
    // This specific original suite qualifies default-profile behavior, including
    // usable HTML when WebGPU is absent; it is not a GPU-only drawing test.
    // All fallback/keyboard/counter/navigation assertions above must pass first.
    s.report["status"] = json!("passed_yt080_default_profile");
    s.report["fallback_checked"] = json!(outcome == "unavailable");
    s.report["gpu_acceptance"] = json!(outcome == "ready");
    Ok(())
}

#[cfg(test)]
mod parent_suite_cleanup {
    use super::*;

    fn scene<'a>(out: &'a Path, temporary: tempfile::TempDir) -> Scene<'a> {
        Scene {
            root: out,
            out,
            assets: out.join("unused"),
            base: String::new(),
            manifest: Value::Null,
            report: json!({"status":"passed","cleanup":{},"input_hashes":{}}),
            browser: None,
            server: None,
            temporary: Some(temporary),
            deadline: Instant::now() + Duration::from_secs(1),
            firefox: true,
        }
    }

    #[test]
    fn owned_directory_failure_reaches_final_return_125_and_sanitized_receipt() {
        let out = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir_in(out.path()).unwrap();
        let path = private.path().to_owned();
        fs::remove_dir(&path).unwrap();
        fs::write(&path, "private raw diagnostic").unwrap();
        let native_receipt = b"{\"native_exit_code\":null,\"signal\":15}";
        fs::write(out.path().join("server.exit.json"), native_receipt).unwrap();
        let mut s = scene(out.path(), private);
        let error = s.finish(Ok(())).unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "scene temporary directory cleanup failed");
        assert!(s.temporary.is_none());
        assert_eq!(s.report["status"], "blocked_cleanup");
        assert_eq!(s.report["cleanup"]["profile_removed"], false);
        assert_eq!(s.report["cleanup"]["processes_reaped"], true);
        assert_eq!(
            s.report["cleanup"]["profile_error"],
            "scene temporary directory cleanup failed"
        );
        let bytes = fs::read(out.path().join("result.json")).unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), s.report);
        assert!(
            !String::from_utf8(bytes)
                .unwrap()
                .contains("private raw diagnostic")
        );
        assert!(!error.message.contains(path.to_str().unwrap()));
        assert_eq!(
            fs::read(out.path().join("server.exit.json")).unwrap(),
            native_receipt
        );
    }

    #[test]
    fn directory_error_survives_primary_timeout_and_report_failure() {
        let out = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir_in(out.path()).unwrap();
        fs::remove_dir(private.path()).unwrap();
        fs::write(private.path(), "fixture").unwrap();
        fs::create_dir(out.path().join("result.json")).unwrap();
        let mut s = scene(out.path(), private);
        let error = s
            .finish(Err(tooling::Error {
                code: 124,
                message: "primary timeout".into(),
            }))
            .unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "scene temporary directory cleanup failed");
        assert_ne!(s.report["status"], "passed");
        assert_eq!(s.report["error"], "primary timeout");
        assert!(s.temporary.is_none());
    }

    #[test]
    fn required_receipt_failure_returns_125_after_owned_directory_removal() {
        let out = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir_in(out.path()).unwrap();
        let path = private.path().to_owned();
        fs::create_dir(out.path().join("result.json")).unwrap();
        let mut s = scene(out.path(), private);
        let error = s.finish(Ok(())).unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "scene report write failed");
        assert_eq!(s.report["status"], "blocked_report");
        assert!(!path.exists());
    }

    #[test]
    fn successful_cleanup_preserves_primary_status_and_input_identity_gate() {
        for code in [0, 1, 23, 124, 125, 130] {
            let out = tempfile::tempdir().unwrap();
            let private = tempfile::tempdir_in(out.path()).unwrap();
            let path = private.path().to_owned();
            let mut s = scene(out.path(), private);
            let primary = if code == 0 {
                Ok(())
            } else {
                Err(tooling::Error {
                    code,
                    message: "primary diagnostic".into(),
                })
            };
            let result = s.finish(primary);
            assert!(!path.exists());
            assert_eq!(s.report["cleanup"]["profile_removed"], true);
            assert_eq!(s.report["cleanup"]["errors"], json!([]));
            if code == 0 {
                result.unwrap();
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, code);
                assert_eq!(error.message, "primary diagnostic");
                assert_eq!(s.report["status"], "failed");
            }
        }
        let out = tempfile::tempdir().unwrap();
        let private = tempfile::tempdir_in(out.path()).unwrap();
        let mut s = scene(out.path(), private);
        s.report["input_hashes"] = json!({"missing-input":"not-the-hash"});
        let error = s.finish(Ok(())).unwrap_err();
        assert_eq!(error.code, 1);
        assert_eq!(error.message, "scene inputs changed during run");
        assert_eq!(s.report["status"], "failed");
    }
}
