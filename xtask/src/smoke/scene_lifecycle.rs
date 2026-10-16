//! YT-076 resize/suspension, real loss and held acquisition/validation lifetimes.
use super::{require, scene::Scene};
use crate::tooling::Result;
use serde_json::json;
fn n(v: &serde_json::Value, k: &str) -> u64 {
    v[k].as_u64().unwrap_or(0)
}
pub fn run(s: &mut Scene<'_>) -> Result<()> {
    s.report["fault_policy"] = json!(
        "Controlled real device.destroy() triggers real loss promise; injected loss, not hardware failure"
    );
    s.fresh("lifecycle")?;
    s.real_adapter()?;
    s.exec(r#"window.__yt076Devices=[];window.__yt076Losses=0;
const request=GPUAdapter.prototype.requestDevice;
GPUAdapter.prototype.requestDevice=async function(...args){const d=await request.apply(this,args);window.__yt076Devices.push(d);d.lost.then(()=>window.__yt076Losses++);return d;};"#)?;
    s.activate()?;
    let before = s.snap("initial owner")?;
    require(
        n(&before, "draws") >= 1
            && before["configurations"] == 1
            && before["rafPending"] == false
            && before["ownedBufferBytes"] == 3840
            && before["observers"] == 3
            && before["listeners"] == 3,
        "initial lifecycle ownership counters",
    )?;
    s.click_id("activate-scene")?;
    s.frames()?;
    let idle = s.snap("static idle")?;
    require(
        idle["draws"] == before["draws"] && idle["starts"] == 1,
        "single owner demand-only idle",
    )?;
    s.record("single owner and demand-only static idle");
    s.exec("const c=document.getElementById('scene-canvas');c.style.width='450px';c.style.height='300px';c.style.width='480px';c.style.height='320px';for(let i=0;i<12;i++)window.__yt075Scene.draw();")?;
    s.frames()?;
    let resized = s.snap("coalesced resize")?;
    require(
        n(&resized, "configurations") == n(&before, "configurations") + 1
            && n(&resized, "coalesced") >= 11
            && resized["rafPending"] == false,
        "coalesced latest resize",
    )?;
    s.exec("const c=document.getElementById('scene-canvas');for(let i=0;i<12;i++){c.style.width='480px';window.__yt075Scene.draw()}")?;
    s.frames()?;
    require(
        s.snap("same size")?["configurations"] == resized["configurations"],
        "no repeated same-size configure",
    )?;
    s.record("coalesced latest resize and repeated same-size configuration");
    s.exec("Object.defineProperty(window,'devicePixelRatio',{value:3,configurable:true});window.dispatchEvent(new Event('resize'))")?;
    s.frames()?;
    let dpr = s.snap("injected DPR 3 bounded to 2")?;
    require(
        dpr["dpr"] == 2
            && dpr["backing"]
                == json!({"width":960.min(n(&dpr,"deviceLimit")),"height":640.min(n(&dpr,"deviceLimit"))}),
        "CSS/backing/DPR clamp",
    )?;
    s.pixels("resized-dpr", None, false)?;
    s.record("CSS/backing/DPR distinction; explicitly injected DPR");
    s.exec("document.getElementById('scene-canvas').style.height='0px'")?;
    s.state("suspended")?;
    let zero = s.snap("zero-sized target")?;
    s.frames()?;
    require(
        s.snap("zero idle")?["draws"] == zero["draws"] && zero["rafPending"] == false,
        "zero size idle",
    )?;
    s.exec("document.getElementById('scene-canvas').style.height='320px'")?;
    s.state("ready")?;
    s.record("zero-size suspension and resume");
    s.exec("window.__yt076Parent=document.getElementById('scene-canvas').parentNode;window.__yt076Canvas=document.getElementById('scene-canvas');window.__yt076Canvas.remove();")?;
    s.state("suspended")?;
    let detached = s.snap("detached")?;
    s.frames()?;
    require(
        s.snap("detached idle")?["draws"] == detached["draws"],
        "detached idle",
    )?;
    s.exec("window.__yt076Parent.append(window.__yt076Canvas);window.__yt076Canvas.scrollIntoView({block:'center'})")?;
    s.state("ready")?;
    s.record("detached target suspension and reattachment");
    s.exec(
        "document.getElementById('scene-canvas').style.marginTop='2000px';window.scrollTo(0,0)",
    )?;
    s.state("suspended")?;
    let off = s.snap("offscreen")?;
    s.frames()?;
    require(
        s.snap("offscreen idle")?["draws"] == off["draws"],
        "offscreen idle",
    )?;
    s.exec("document.getElementById('scene-canvas').style.marginTop='0px';document.getElementById('scene-canvas').scrollIntoView({block:'center'})")?;
    s.state("ready")?;
    s.record("real intersection offscreen pause and resume");
    s.exec(r#"window.__yt076Hidden=[];window.__yt076Visibility=()=>{if(document.hidden){const first=window.__yt075Scene.snapshot();setTimeout(()=>window.__yt076Hidden.push({first,later:window.__yt075Scene.snapshot()}),150);}};document.addEventListener('visibilitychange',window.__yt076Visibility);"#)?;
    let original = s.client()?.firefox()?.command("/window", "GET", None)?;
    let tab =
        s.client()?
            .firefox()?
            .command("/window/new", "POST", Some(&json!({"type":"tab"})))?["handle"]
            .clone();
    s.client()?
        .firefox()?
        .command("/window", "POST", Some(&json!({"handle":tab})))?;
    s.async_js("const done=arguments[arguments.length-1];setTimeout(()=>done(null),450)")?;
    s.client()?.firefox()?.command("/window", "DELETE", None)?;
    s.client()?
        .firefox()?
        .command("/window", "POST", Some(&json!({"handle":original})))?;
    s.state("ready")?;
    let hidden=s.exec("document.removeEventListener('visibilitychange',window.__yt076Visibility);return window.__yt076Hidden")?;
    let last = hidden
        .as_array()
        .and_then(|a| a.last())
        .cloned()
        .unwrap_or_default();
    require(
        last["first"]["state"] == "suspended"
            && last["later"]["draws"] == last["first"]["draws"]
            && last["later"]["rafPending"] == false,
        "real background visibility idle",
    )?;
    s.report["visibility_proof"] = hidden;
    s.record("real background-tab visibility pause and resume");
    let lost_before = s.snap("before controlled real device destruction")?;
    s.exec("window.__yt076Devices[0].destroy()")?;
    s.state("lost")?;
    let lost = s.snap("real loss promise")?;
    require(
        lost["fault"] == "device-lost"
            && lost["ownedBufferBytes"] == 0
            && lost["ownedSurfaces"] == 0
            && lost["cpuRecordBytes"] == lost_before["cpuRecordBytes"]
            && lost["cpuRecordBytes"] == 3840
            && lost["observers"] == 0
            && lost["listeners"] == 0
            && lost["rafPending"] == false,
        "lost resources released CPU records retained",
    )?;
    s.frames()?;
    require(
        s.snap("loss idle")?["starts"] == lost_before["starts"],
        "loss does not self retry",
    )?;
    s.check("window.__yt076Losses===1")?;
    s.click_id("counter-increment")?;
    s.check("document.getElementById('counter-value').textContent==='3' && document.querySelectorAll('#records li').length===12")?;
    s.activate()?;
    let recovered = s.snap("explicit recovery")?;
    require(
        n(&recovered, "generation") > n(&lost_before, "generation")
            && n(&recovered, "starts") == n(&lost_before, "starts") + 1
            && recovered["ownedBufferBytes"] == 3840,
        "one explicit recovery owner",
    )?;
    s.pixels("recovered", None, false)?;
    s.record("injected real device loss, retained HTML/CPU records, sibling and explicit recovery");
    s.click_id("dispose-scene")?;
    let disposed = s.snap("dispose")?;
    require(
        disposed["state"] == "disposed"
            && disposed["ownedBufferBytes"] == 0
            && disposed["ownedSurfaces"] == 0
            && disposed["listeners"] == 0
            && disposed["observers"] == 0
            && disposed["rafPending"] == false,
        "disposed counters",
    )?;
    s.activate()?;
    require(
        n(&s.snap("remount")?, "generation") > n(&recovered, "generation"),
        "remount generation advances",
    )?;
    s.click_id("dispose-scene")?;
    s.record("owned disposal and remount");
    for phase in ["acquisition", "validation"] {
        s.fresh(&format!("late_{phase}"))?;
        s.exec(&HELD.replace("PHASE", &json!(phase).to_string()))?;
        s.click_id("activate-scene")?;
        s.until("window.__yt076Held.held", "held real resource", 35)?;
        s.click_id("activate-scene")?;
        require(
            s.snap(&format!("pending duplicate {phase}"))?["starts"] == 1,
            "one pending initialization",
        )?;
        s.click_id("dispose-scene")?;
        s.activate()?;
        let replacement = s.snap(&format!("replacement {phase}"))?;
        s.exec("window.__yt076Held.release()")?;
        s.until("window.__yt076Held.destroyed===1 && window.__yt075Scene.snapshot().pendingInitializations===0","late resource destruction",25)?;
        let after = s.snap(&format!("late completion {phase}"))?;
        require(
            after["generation"] == replacement["generation"]
                && after["starts"] == 2
                && after["configurations"] == replacement["configurations"]
                && after["state"] == "ready"
                && after["lateSettled"] == 1,
            "late completion cannot replace current owner",
        )?;
        require(
            s.eval("window.__yt076Held.buffersDestroyed")?
                == if phase == "validation" {
                    json!(1)
                } else {
                    json!(0)
                },
            "held buffer destruction count",
        )?;
        s.click_id("counter-increment")?;
        s.check("document.getElementById('counter-value').textContent==='3'")?;
        s.click_id("dispose-scene")?;
        let final_state = s.snap(&format!("final teardown {phase}"))?;
        require(
            final_state["lastRetired"]["resourcesReleased"] == true
                && final_state["pendingInitializations"] == 0
                && final_state["ownedBufferBytes"] == 0
                && final_state["listeners"] == 0
                && final_state["observers"] == 0,
            "late teardown owns no resources",
        )?;
        s.record(&format!(
            "late real {phase} destroyed without replacing current owner"
        ));
    }
    s.report["status"] = json!("passed_yt076_lifecycle");
    Ok(())
}
const HELD: &str = r#"window.__yt076Held={calls:0,held:false,destroyed:0,buffersDestroyed:0};
const request=GPUAdapter.prototype.requestDevice;
GPUAdapter.prototype.requestDevice=async function(...args){
 const first=++window.__yt076Held.calls===1;const d=await request.apply(this,args);if(!first)return d;
 const destroy=d.destroy.bind(d);d.destroy=()=>{window.__yt076Held.destroyed++;destroy()};
 const create=d.createBuffer.bind(d);d.createBuffer=(...args)=>{const b=create(...args);const destroy=b.destroy.bind(b);b.destroy=()=>{window.__yt076Held.buffersDestroyed++;destroy()};return b;};
 if(PHASE==='acquisition')return await new Promise(resolve=>{window.__yt076Held.held=true;window.__yt076Held.release=()=>resolve(d)});
 const pop=d.popErrorScope.bind(d);d.popErrorScope=async()=>{const result=await pop();return await new Promise(resolve=>{window.__yt076Held.held=true;window.__yt076Held.release=()=>resolve(result)});};return d;
};"#;
