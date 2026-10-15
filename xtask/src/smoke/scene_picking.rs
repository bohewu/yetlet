//! YT-078 live W3C input/selection, stale picks and BiDi portrait geometry.
use super::{require, scene::Scene};
use crate::tooling::{Result, fail};
use serde_json::{Value, json};
use std::fs;
impl Scene<'_> {
    pub fn observe(&mut self, selector: &str, label: &str) -> Result<(String, Value)> {
        let key = self.client()?.firefox()?.element(selector)?;
        let rect =
            self.client()?
                .firefox()?
                .command(&format!("/element/{key}/rect"), "GET", None)?;
        let data=self.exec(&format!(r#"const t=document.querySelector({}),c=document.getElementById('scene-canvas');
return {{url:location.href,connected:t.isConnected,targetText:t.textContent,label:t.getAttribute('aria-label'),describedBy:t.getAttribute('aria-describedby'),disabled:t.disabled??false,
scene:window.__yt075Scene?.snapshot()??null,selection:window.__yt078Selection?.snapshot()??null,detail:document.getElementById('scene-selection')?.textContent??null,href:document.getElementById('scene-detail-link')?.getAttribute('href')??null,
records:document.querySelectorAll('#records li').length,counter:document.getElementById('counter-value')?.textContent??null,
css:c?{{left:c.getBoundingClientRect().left,top:c.getBoundingClientRect().top,width:c.getBoundingClientRect().width,height:c.getBoundingClientRect().height}}:null}};"#,json!(selector)))?;
        require(
            data["connected"] == true && data["disabled"] == false,
            "fresh target connected and enabled",
        )?;
        self.report["snapshots"].as_array_mut().unwrap().push(json!({"label":label,"selector":selector,"webdriver_target":key,"rect":rect,"value":data}));
        Ok((key, data))
    }
    pub fn action(
        &mut self,
        selector: &str,
        label: &str,
        operation: impl FnOnce(&mut Self, &str, &Value) -> Result<Value>,
    ) -> Result<Value> {
        let (key, before) = self.observe(selector, &format!("{label} before"))?;
        let result = operation(self, &key, &before)?;
        self.frames()?;
        let (_, after) = self.observe(selector, &format!("{label} after"))?;
        self.report["checks"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name":label,"action_result":result,"status":"executed"}));
        self.report["actions"]
            .as_array_mut()
            .unwrap()
            .push(json!(label));
        Ok(after)
    }
    pub fn native_click(&mut self, selector: &str, label: &str) -> Result<Value> {
        self.action(selector, label, |s, key, _| {
            s.client()?.firefox()?.command(
                &format!("/element/{key}/click"),
                "POST",
                Some(&json!({})),
            )
        })
    }
    pub fn script(&mut self, selector: &str, label: &str, code: &str) -> Result<Value> {
        self.action(selector, label, |s, _, _| s.exec(code))
    }
    pub fn native_keys(&mut self, selector: &str, label: &str, text: &str) -> Result<Value> {
        self.action(selector,label,|s,key,_|s.client()?.firefox()?.command(&format!("/element/{key}/value"),"POST",Some(&json!({"text":text,"value":text.chars().map(|c|c.to_string()).collect::<Vec<_>>()}))))
    }
    pub fn ready(&mut self) -> Result<()> {
        self.script(
            "#scene-canvas",
            "restore visible canvas before readiness",
            "document.getElementById('scene-canvas').scrollIntoView({block:'center'})",
        )?;
        self.until("window.__yt075Scene.snapshot().state==='ready' && window.__yt075Scene.snapshot().draws>=1","drawn GPU ready",35)?;
        self.frames()
    }
    pub fn pointer(&mut self, id: u64, label: &str) -> Result<()> {
        self.script(
            "#scene-canvas",
            &format!("visible input target {label}"),
            "document.getElementById('scene-canvas').scrollIntoView({block:'center'})",
        )?;
        let after=self.action("#scene-canvas",label,|s,_,before|{
            let (x,y)=pointer_center(&before["css"],id)?;
            s.client()?.firefox()?.command("/actions","POST",Some(&json!({"actions":[{"type":"pointer","id":"yt085-mouse","parameters":{"pointerType":"mouse"},"actions":[{"type":"pointerMove","duration":0,"origin":"viewport","x":x,"y":y},{"type":"pointerDown","button":0},{"type":"pointerUp","button":0}]}]})))
        })?;
        require(
            after["selection"]["selectedRecord"] == id
                && after["detail"]
                    .as_str()
                    .is_some_and(|v| v.contains(&format!("{id:03}")))
                && after["href"] == format!("/scene?record={id}"),
            "actual mouse stable record selection",
        )
    }
    pub fn scriptless(&mut self) -> Result<()> {
        let manifest = self.assets.join("manifest.json");
        let saved = fs::read(&manifest)?;
        fs::remove_file(&manifest)?;
        let r = (|| {
            let url = format!("{}/scene?q=done&page=1&record=38", self.base);
            let nojs = self.action("#query", "scriptless SSR search/page/detail", |s, _, _| {
                s.client()?.navigate(&url)?;
                Ok(Value::Null)
            })?;
            require(
                nojs["records"] == 12 && nojs["counter"] == "2",
                "scriptless SSR record data",
            )?;
            self.check("!document.querySelector('script') && document.getElementById('matches').textContent.includes('40 matches; page 2 of 4') && document.getElementById('detail').textContent.includes('Work record 038')")?;
            let opened = self.native_click("#records a", "scriptless real record navigation")?;
            require(
                opened["url"]
                    .as_str()
                    .is_some_and(|u| u.contains("record=38")),
                "scriptless record href",
            )?;
            self.check(
                "document.getElementById('detail').textContent.includes('Work record 038')",
            )?;
            self.report["scriptless_ssr"] = json!(
                "No script tags/assets; actual SSR search/page/detail and link navigation; not a browser JS-disabled preference claim"
            );
            Ok(())
        })();
        fs::write(&manifest, saved)?;
        r
    }
}
pub fn pointer_center(css: &Value, id: u64) -> Result<(i64, i64)> {
    require((1..=120).contains(&id), "picking ID range")?;
    let get = |key| {
        css[key]
            .as_f64()
            .filter(|v| v.is_finite())
            .ok_or_else(|| fail("picking CSS geometry"))
    };
    let (left, top, width, height) = (get("left")?, get("top")?, get("width")?, get("height")?);
    require(width > 0. && height > 0., "positive picking geometry")?;
    Ok((
        (left + ((id - 1) % 12) as f64 * width / 12. + width / 24.).round_ties_even() as i64,
        (top + ((id - 1) / 12) as f64 * height / 10. + height / 20.).round_ties_even() as i64,
    ))
}
pub fn run(s: &mut Scene<'_>) -> Result<()> {
    s.report["browser_process_bound_seconds"] = json!(240);
    s.report["test_deadline_seconds"] = json!(180);
    s.fresh("picking")?;
    s.action("body", "desktop browser viewport", |s, _, _| {
        s.client()?.firefox()?.command(
            "/window/rect",
            "POST",
            Some(&json!({"width":1100,"height":1000})),
        )
    })?;
    s.real_adapter()?;
    s.native_click("#activate-scene", "explicit GPU activation")?;
    s.ready()?;
    s.pixels("initial", None, false)?;
    s.pointer(1, "actual mouse selects ID 1")?;
    s.pointer(120, "actual mouse selects ID 120")?;
    s.native_click("#scene-navigate", "focus labeled keyboard selection")?;
    for (key, label) in [
        ("\u{e011}", "Home chooses ID 1"),
        ("\u{e014}", "Right chooses ID 2"),
        ("\u{e015}", "Down chooses ID 14"),
    ] {
        s.native_keys("#scene-navigate", label, key)?;
    }
    let selected = s.native_keys("#scene-navigate", "Enter selects ID 14", "\u{e007}")?;
    require(
        selected["selection"]["selectedRecord"] == 14 && selected["href"] == "/scene?record=14",
        "keyboard ID14",
    )?;
    s.native_keys("#scene-navigate", "End chooses final ID", "\u{e010}")?;
    require(
        s.native_keys("#scene-navigate", "Space selects final ID", " ")?["selection"]["selectedRecord"]
            == 120,
        "keyboard ID120",
    )?;
    let reset = s.native_click("#scene-reset", "keyboard alternative reset")?;
    require(
        reset["selection"]["selectedRecord"].is_null() && reset["href"].is_null(),
        "reset clears detail",
    )?;
    s.script("#scene-canvas","resize CSS with simulated DPR 3 clamped to 2","const c=document.getElementById('scene-canvas');c.style.width='300px';c.style.height='200px';Object.defineProperty(window,'devicePixelRatio',{value:3,configurable:true});window.dispatchEvent(new Event('resize'))")?;
    s.ready()?;
    let (_, mapped) = s.observe("#scene-canvas", "DPR mapping assertion")?;
    require(
        mapped["scene"]["dpr"] == 2
            && mapped["scene"]["backing"] == json!({"width":600,"height":400}),
        "CSS to backing geometry",
    )?;
    s.pixels("resized-injected-dpr", None, false)?;
    s.pointer(13, "actual mouse after CSS/DPR remapping selects ID 13")?;
    let touch = r#"const c=document.getElementById('scene-canvas'),b=c.getBoundingClientRect();const init={bubbles:true,pointerId:78,pointerType:'touch',isPrimary:true,button:0,clientX:b.left+b.width*1.5/12,clientY:b.top+b.height*.5/10};c.dispatchEvent(new PointerEvent('pointerdown',init));MID;c.dispatchEvent(new PointerEvent('pointerup',init));"#;
    require(
        s.script(
            "#scene-canvas",
            "SIMULATED touch tap selects ID 2",
            &touch.replace("MID", ""),
        )?["selection"]["selectedRecord"]
            == 2,
        "simulated tap",
    )?;
    let cancelled = s.script(
        "#scene-canvas",
        "SIMULATED pointercancel preserves ID 2",
        &touch.replace("1.5/12", "2.5/12").replace(
            "MID",
            "c.dispatchEvent(new PointerEvent('pointercancel',init))",
        ),
    )?;
    require(
        cancelled["selection"]["selectedRecord"] == 2
            && cancelled["selection"]["tapPending"] == false,
        "pointercancel clears pending",
    )?;
    let scrolled = s.script(
        "#scene-canvas",
        "SIMULATED scroll preserves ID 2",
        &touch
            .replace("1.5/12", "2.5/12")
            .replace("MID", "window.dispatchEvent(new Event('scroll'))"),
    )?;
    require(
        scrolled["selection"]["selectedRecord"] == 2,
        "scroll cancels tap",
    )?;
    let moved = s.script(
        "#scene-canvas",
        "SIMULATED move preserves ID 2",
        &touch.replace("1.5/12", "2.5/12").replace(
            "MID",
            "c.dispatchEvent(new PointerEvent('pointermove',{...init,clientX:init.clientX+20}))",
        ),
    )?;
    require(
        moved["selection"]["selectedRecord"] == 2,
        "move cancels tap",
    )?;
    s.check(
        "getComputedStyle(document.getElementById('scene-canvas')).touchAction.includes('pan-y')",
    )?;
    s.script("#scene-canvas","capture deferred old-data pick","const b=document.getElementById('scene-canvas').getBoundingClientRect();window.__yt078Old=window.__yt078Selection.capturePick(b.left+b.width/24,b.top+b.height/20)")?;
    s.action("#scene-canvas","prepare same-ID replacement records",|s,_,_|s.async_js("const done=arguments[arguments.length-1];window.__yt075Scene.records(120,0).then(data=>{window.__yt078Data=data;done(null)});"))?;
    let replaced=s.script("#scene-canvas","replace and reject stale-version pick","window.__yt075Scene.replace(window.__yt078Data);return window.__yt078Selection.commitPick(window.__yt078Old)")?;
    require(
        replaced["selection"]["selectedRecord"].is_null()
            && replaced["selection"]["rejectedPicks"].as_u64()
                > scrolled["selection"]["rejectedPicks"].as_u64(),
        "reject stale data version",
    )?;
    s.pixels("replaced", None, false)?;
    s.script("#scene-canvas","capture old-generation pick","const b=document.getElementById('scene-canvas').getBoundingClientRect();window.__yt078Old=window.__yt078Selection.capturePick(b.left+b.width/24,b.top+b.height/20)")?;
    let disposed = s.native_click("#dispose-scene", "owned disposal")?;
    require(
        disposed["scene"]["ownedSurfaces"] == 0 && disposed["scene"]["ownedBufferBytes"] == 0,
        "disposed GPU counters",
    )?;
    require(
        s.script(
            "#scene-canvas",
            "reject deferred pick after disposal",
            "return window.__yt078Selection.commitPick(window.__yt078Old)",
        )?["selection"]["selectedRecord"]
            .is_null(),
        "disposed pick rejected",
    )?;
    s.native_click("#activate-scene", "explicit replacement generation")?;
    s.ready()?;
    require(
        s.script(
            "#scene-canvas",
            "reject old generation after replacement",
            "return window.__yt078Selection.commitPick(window.__yt078Old)",
        )?["selection"]["selectedRecord"]
            .is_null(),
        "old generation rejected",
    )?;
    s.action(
        "#scene-canvas",
        "390 CSS-pixel portrait viewport via BiDi",
        |s, _, _| s.portrait(),
    )?;
    s.script("#scene-canvas","portrait responsive aspect layout","const c=document.getElementById('scene-canvas');c.style.width='';c.style.height='';c.scrollIntoView({block:'center'})")?;
    s.ready()?;
    s.pixels("portrait", None, false)?;
    s.pointer(3, "portrait actual mouse selects ID 3")?;
    let (_, portrait) = s.observe("#scene-navigate", "portrait labels/data/actions")?;
    require(
        portrait["records"] == 12 && portrait["describedBy"] == "selection-help",
        "portrait semantic navigation",
    )?;
    s.report["portrait_layout"]=s.eval("({innerWidth,innerHeight,outerWidth,scrollWidth:document.documentElement.scrollWidth,records:document.querySelectorAll('#records li').length})")?;
    s.check("innerWidth===390 && innerHeight>390 && document.documentElement.scrollWidth<=390 && document.querySelectorAll('#records li').length===12")?;
    let (key, _) = s.observe("#scene-navigate", "accessible keyboard name")?;
    let name =
        s.client()?
            .firefox()?
            .command(&format!("/element/{key}/computedlabel"), "GET", None)?;
    require(
        name.as_str().is_some_and(|v| v.contains("Select record")),
        "keyboard accessible name",
    )?;
    s.report["accessible_navigation_name"] = name;
    let (key, _) = s.observe("#scene-canvas", "accessible canvas name")?;
    require(
        s.client()?
            .firefox()?
            .command(&format!("/element/{key}/computedlabel"), "GET", None)?
            .as_str()
            .is_some_and(|v| v.contains("HTML record list")),
        "canvas accessible name",
    )?;
    let motion=s.exec("const c=document.getElementById('scene-canvas'),s=getComputedStyle(c);return {matches:matchMedia('(prefers-reduced-motion: reduce)').matches,animation:s.animationName,transition:s.transitionDuration,mediaRule:[...document.styleSheets].some(sheet=>[...sheet.cssRules].some(rule=>rule.conditionText==='(prefers-reduced-motion: reduce)'))}")?;
    require(
        motion["animation"] == "none"
            && motion["transition"] == "0s"
            && motion["mediaRule"] == true,
        "reduced-motion static rendering",
    )?;
    s.report["reduced_motion"] = motion;
    s.native_click("#dispose-scene", "portrait final GPU teardown")?;
    let (_, teardown) = s.observe("#scene-canvas", "teardown counters")?;
    require(
        teardown["scene"]["lastRetired"]["resourcesReleased"] == true
            && teardown["scene"]["observers"] == 0
            && teardown["scene"]["listeners"] == 0
            && teardown["scene"]["rafPending"] == false,
        "portrait resource teardown",
    )?;
    s.fresh("unavailable")?;
    s.script(
        "#activate-scene",
        "INJECT unavailable GPU capability",
        "Object.defineProperty(navigator,'gpu',{value:undefined,configurable:true})",
    )?;
    s.native_click("#activate-scene", "activate unavailable fallback")?;
    s.until(
        "window.__yt075Scene.snapshot().state==='unavailable'",
        "unavailable GPU state",
        15,
    )?;
    s.native_click("#scene-navigate", "fallback keyboard selection")?;
    s.native_keys("#scene-navigate", "fallback End selection", "\u{e010}")?;
    let fallback = s.native_keys("#scene-navigate", "fallback Enter detail", "\u{e007}")?;
    require(
        fallback["selection"]["selectedRecord"] == 120 && fallback["href"] == "/scene?record=120",
        "fallback selection",
    )?;
    require(
        s.native_click("#counter-increment", "independent counter without GPU")?["counter"] == "3",
        "fallback sibling",
    )?;
    let before = fallback["selection"].clone();
    s.native_keys("#query", "native text search input", "done")?;
    let unchanged = s.native_keys(
        "#query",
        "search ArrowLeft does not navigate scene",
        "\u{e012}",
    )?;
    require(
        unchanged["selection"]["focusedRecord"] == before["focusedRecord"],
        "input arrow scoped",
    )?;
    let ime=s.script("#query","SIMULATED composing key is not intercepted","const q=document.getElementById('query');const e=new KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true,cancelable:true,isComposing:true});q.dispatchEvent(new CompositionEvent('compositionstart',{data:'中'}));q.dispatchEvent(e);q.dispatchEvent(new CompositionEvent('compositionend',{data:'中'}));return e.defaultPrevented;")?;
    require(
        ime["selection"]["focusedRecord"] == before["focusedRecord"],
        "simulated IME does not navigate scene",
    )?;
    s.scriptless()?;
    s.report["status"] = json!("passed_yt078_picking");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_pointer_grid_mapping_rejects_invalid_ids_and_geometry() {
        let css = json!({"left":10,"top":20,"width":600,"height":400});
        assert_eq!(pointer_center(&css, 1).unwrap(), (35, 40));
        assert_eq!(pointer_center(&css, 120).unwrap(), (585, 400));
        assert!(pointer_center(&css, 0).is_err());
        assert!(pointer_center(&css, 121).is_err());
        assert!(pointer_center(&json!({"left":0,"top":0,"width":0,"height":20}), 1).is_err());
        assert!(pointer_center(&json!({}), 1).is_err());
    }
}
