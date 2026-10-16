//! YT-079 extracted source-bound renderer; YT-081 refinement stays explicit opt-in.
use super::{require, scene::Scene};
use crate::tooling::{self, Result, fail};
use serde_json::json;
use std::fs;
pub fn run(s: &mut Scene<'_>) -> Result<()> {
    let refinement = std::env::var("YT081_WGPU_API_REFINEMENT").as_deref() == Ok("1");
    if refinement {
        s.report["task_id"] = json!("YT-081");
    }
    s.report["browser_process_bound_seconds"] = json!(240);
    s.report["test_deadline_seconds"] = json!(180);
    s.fresh("candidate")?;
    s.action("body", "desktop viewport", |s, _, _| {
        s.client()?.firefox()?.command(
            "/window/rect",
            "POST",
            Some(&json!({"width":1100,"height":1000})),
        )
    })?;
    s.real_adapter()?;
    s.native_click("#activate-scene", "explicit extracted renderer activation")?;
    s.ready()?;
    s.pixels("initial", None, false)?;
    s.pointer(1, "actual mouse known slot 1")?;
    s.native_click("#scene-navigate", "focus keyboard alternative")?;
    s.native_keys("#scene-navigate", "keyboard Home", "\u{e011}")?;
    s.native_keys("#scene-navigate", "keyboard Right", "\u{e014}")?;
    let chosen = s.native_keys(
        "#scene-navigate",
        "keyboard Enter selects slot 2",
        "\u{e007}",
    )?;
    require(
        chosen["selection"]["selectedRecord"] == 2
            && chosen["href"] == "/scene?record=2"
            && chosen["describedBy"] == "selection-help",
        "candidate keyboard selection",
    )?;
    s.action("#scene-canvas","prepare app-owned replacement",|s,_,_|s.async_js("const done=arguments[arguments.length-1];window.__yt075Scene.records(120,1).then(data=>{window.__yt079Data=data;done(null)});"))?;
    s.script("#scene-canvas","capture old-data pick","const b=document.getElementById('scene-canvas').getBoundingClientRect();window.__yt079Pick=window.__yt078Selection.capturePick(b.left+b.width/24,b.top+b.height/20)")?;
    let replaced=s.script("#scene-canvas","replace and reject stale pick","window.__yt075Scene.replace(window.__yt079Data);return window.__yt078Selection.commitPick(window.__yt079Pick)")?;
    require(
        replaced["selection"]["selectedRecord"].is_null()
            && replaced["selection"]["rejectedPicks"].as_u64()
                > chosen["selection"]["rejectedPicks"].as_u64(),
        "candidate stale pick rejected",
    )?;
    let replacement_colors: Vec<_> = (0..120).map(|i| (i + 1) % 3).collect();
    s.pixels("replacement", Some(&replacement_colors), false)?;
    let before = s.eval("window.__yt075Scene.snapshot().metrics")?;
    s.action("#scene-canvas","prepare 12-record status update",|s,_,_|s.async_js("const done=arguments[arguments.length-1];window.__yt075Scene.patch(window.__yt079Data,0,12,1,1).then(data=>{window.__yt079Patch=data;done(null)});"))?;
    s.script(
        "#scene-canvas",
        "upload bounded status patch",
        "window.__yt075Scene.update(0,window.__yt079Patch)",
    )?;
    let updated_colors: Vec<_> = (0..120)
        .map(|i| (i + if i < 12 { 2 } else { 1 }) % 3)
        .collect();
    s.pixels("updated", Some(&updated_colors), false)?;
    let after = s.eval("window.__yt075Scene.snapshot().metrics")?;
    let metric = |v: &serde_json::Value, i: usize| v[i].as_u64().unwrap_or(0);
    require(
        metric(&after, 0) == metric(&before, 0) + 12 * 32
            && metric(&after, 1) == metric(&before, 1) + 1
            && after[9] == 120 * 32
            && after[11] == 1,
        "bounded upload byte/call/ownership metrics",
    )?;
    s.pointer(120, "known pick after update")?;
    s.action(
        "#scene-canvas",
        "390 CSS-pixel narrow viewport via BiDi",
        |s, _, _| s.portrait(),
    )?;
    s.script(
        "#scene-canvas",
        "narrow responsive target",
        "document.getElementById('scene-canvas').scrollIntoView({block:'center'})",
    )?;
    s.ready()?;
    s.report["narrow_layout"]=s.eval("({innerWidth,innerHeight,scrollWidth:document.documentElement.scrollWidth,records:document.querySelectorAll('#records li').length})")?;
    s.check("innerWidth===390 && document.documentElement.scrollWidth<=390 && document.querySelectorAll('#records li').length===12")?;
    s.pointer(3, "narrow known pick")?;
    s.native_click("#dispose-scene", "candidate teardown")?;
    let (_, disposed) = s.observe("#scene-canvas", "disposed counters")?;
    require(
        disposed["scene"]["lastRetired"]["resourcesReleased"] == true
            && disposed["scene"]["ownedSurfaces"] == 0
            && disposed["scene"]["ownedBufferBytes"] == 0
            && disposed["scene"]["observers"] == 0
            && disposed["scene"]["listeners"] == 0
            && disposed["scene"]["rafPending"] == false,
        "candidate disposed resources",
    )?;
    s.fresh("loss")?;
    s.script("#activate-scene","capture acquired devices for controlled loss",r#"window.__yt079Devices=[];const request=GPUAdapter.prototype.requestDevice;GPUAdapter.prototype.requestDevice=async function(...args){const d=await request.apply(this,args);window.__yt079Devices.push(d);return d;};"#)?;
    s.native_click("#activate-scene", "activate loss sample")?;
    s.ready()?;
    let (_, before_loss) = s.observe("#scene-canvas", "before controlled loss")?;
    s.script(
        "#scene-canvas",
        "CONTROLLED real device destroy",
        "window.__yt079Devices[0].destroy()",
    )?;
    s.until(
        "window.__yt075Scene.snapshot().state==='lost'",
        "controlled loss fallback",
        20,
    )?;
    let (_, lost) = s.observe("#scene-canvas", "lost fallback counters")?;
    require(
        lost["scene"]["fault"] == "device-lost"
            && lost["scene"]["ownedSurfaces"] == 0
            && lost["scene"]["ownedBufferBytes"] == 0
            && lost["scene"]["starts"] == before_loss["scene"]["starts"]
            && lost["records"] == 12
            && lost["scene"]["listeners"] == 0
            && lost["scene"]["observers"] == 0
            && lost["scene"]["rafPending"] == false,
        "candidate loss releases ownership without retry",
    )?;
    require(
        s.native_click("#counter-increment", "counter survives lost renderer")?["counter"] == "3",
        "loss sibling",
    )?;
    s.native_click("#activate-scene", "one explicit loss recovery")?;
    s.ready()?;
    let (_, recovered) = s.observe("#scene-canvas", "fresh recovery generation")?;
    require(
        recovered["scene"]["generation"].as_u64() > before_loss["scene"]["generation"].as_u64()
            && recovered["scene"]["starts"].as_u64()
                == before_loss["scene"]["starts"].as_u64().map(|v| v + 1),
        "candidate explicit recovery generation",
    )?;
    s.pixels("recovered", None, false)?;
    s.native_click("#dispose-scene", "recovery teardown")?;
    s.report["loss_policy"] = json!(
        "Controlled real device.destroy and real loss promise; not physical hardware failure"
    );
    s.fresh("completion-retirement")?;
    s.script(
        "#activate-scene",
        "install pending-completion/resource sample",
        HELD,
    )?;
    s.native_click("#activate-scene", "activate completion sample")?;
    s.ready()?;
    let hook = if refinement { REFINEMENT_HOOK } else { HOOK };
    let digest = s.manifest["scene"]["digest"]
        .as_str()
        .ok_or_else(|| fail("candidate digest"))?;
    let hook_path = s
        .assets
        .join("assets")
        .join(digest)
        .join("yt079-free-hook.js");
    fs::write(&hook_path, hook)?;
    let hook_url = format!("/scene-assets/{digest}/yt079-free-hook.js");
    s.script("#scene-canvas","observe page-world generated wrapper freeing",&format!("const s=document.createElement('script');s.type='module';s.src={};document.head.append(s)",json!(hook_url)))?;
    s.until(
        "window.__yt079Held.hookInstalled===true",
        "owned page-world hook loaded",
        15,
    )?;
    s.report["wrapper_hook"] = json!({"scope":"temporary same-origin page module, not production asset","sha256":tooling::sha(hook.as_bytes()),"strict_csp_unchanged":true});
    s.script("#scene-canvas","start retained completion future","window.__yt075Scene.completion().then(()=>window.__yt079Completion='resolved',error=>window.__yt079Completion=String(error))")?;
    s.until("window.__yt079Held.held", "held real queue completion", 20)?;
    let (_, pending) = s.observe("#scene-canvas", "pending completion owner")?;
    require(
        pending["scene"]["pendingCompletions"] == 1,
        "one pending completion",
    )?;
    s.native_click("#dispose-scene", "retire owner during completion")?;
    s.check("window.__yt079Held.frees===0 && window.__yt079Held.destroyed===1 && window.__yt079Held.buffers===1 && window.__yt079Held.unconfigured===1 && !window.__yt075Scene.snapshot().lastRetired.resourcesReleased")?;
    s.native_click(
        "#activate-scene",
        "replacement while old completion pending",
    )?;
    s.ready()?;
    let (_, replacement) = s.observe("#scene-canvas", "replacement generation ready")?;
    require(
        replacement["scene"]["generation"].as_u64() > pending["scene"]["generation"].as_u64(),
        "replacement generation advanced",
    )?;
    s.script(
        "#scene-canvas",
        "settle retired completion",
        "window.__yt079Held.release()",
    )?;
    s.until(
        "window.__yt079Held.frees===1 && !!window.__yt079Completion",
        "old wrapper freed once",
        15,
    )?;
    s.check("window.__yt079Completion.includes('retired') && window.__yt079Held.destroyed===1 && window.__yt079Held.buffers===1 && window.__yt079Held.unconfigured===1 && window.__yt075Scene.snapshot().lastRetired.resourcesReleased")?;
    let (_, settled) = s.observe(
        "#scene-canvas",
        "replacement unaffected by stale completion",
    )?;
    require(
        settled["scene"]["generation"] == replacement["scene"]["generation"]
            && settled["scene"]["state"] == "ready",
        "candidate replacement unchanged",
    )?;
    s.pixels("after-retired-completion", None, false)?;
    s.report["completion_retirement"]=s.exec("const h=window.__yt079Held;return {held:h.held,destroyed:h.destroyed,buffers:h.buffers,unconfigured:h.unconfigured,frees:h.frees,settlement:window.__yt079Completion}")?;
    s.native_click("#dispose-scene", "completion replacement teardown")?;
    s.check("window.__yt079Held.frees===2 && window.__yt075Scene.snapshot().lastRetired.resourcesReleased")?;
    if refinement {
        let pair =
            s.exec("return {inert:window.__yt081DisposePair,live:window.__yt081DisposeCalls}")?;
        require(
            pair["inert"] == json!([true, false])
                && pair["live"]
                    .as_array()
                    .is_some_and(|v| v.len() >= 2 && v.iter().all(|v| v == true)),
            "actual Wasm idempotent disposal",
        )?;
        s.report["idempotent_dispose"] = pair;
    }
    s.fresh("unavailable")?;
    s.script("#activate-scene","INJECT unavailable GPU capability","window.__yt079GPU=navigator.gpu;Object.defineProperty(navigator,'gpu',{value:undefined,configurable:true})")?;
    s.native_click("#activate-scene", "unavailable fallback")?;
    s.until(
        "window.__yt075Scene.snapshot().state==='unavailable'",
        "unavailable state",
        15,
    )?;
    s.native_click("#scene-navigate", "fallback keyboard")?;
    s.native_keys("#scene-navigate", "fallback End", "\u{e010}")?;
    let fallback = s.native_keys("#scene-navigate", "fallback Enter", "\u{e007}")?;
    require(
        fallback["selection"]["selectedRecord"] == 120
            && fallback["href"] == "/scene?record=120"
            && fallback["records"] == 12,
        "candidate fallback selection",
    )?;
    require(
        s.native_click("#counter-increment", "independent counter without GPU")?["counter"] == "3",
        "candidate fallback sibling",
    )?;
    s.script(
        "#activate-scene",
        "restore injected capability",
        "Object.defineProperty(navigator,'gpu',{value:window.__yt079GPU,configurable:true})",
    )?;
    s.native_click("#activate-scene", "explicit unavailable recovery")?;
    s.ready()?;
    s.native_click("#dispose-scene", "unavailable recovery teardown")?;
    s.scriptless()?;
    s.report["status"] = json!("passed_yt079_candidate");
    Ok(())
}
const HELD: &str = r#"window.__yt079Held={held:false,destroyed:0,buffers:0,unconfigured:0,frees:0};
const request=GPUAdapter.prototype.requestDevice;
GPUAdapter.prototype.requestDevice=async function(...args){const first=!window.__yt079Held.deviceSeen;
 const d=await request.apply(this,args);if(!first)return d;window.__yt079Held.deviceSeen=true;
 const destroy=d.destroy.bind(d);d.destroy=()=>{window.__yt079Held.destroyed++;destroy()};
 const create=d.createBuffer.bind(d);d.createBuffer=(...args)=>{const b=create(...args),destroy=b.destroy.bind(b);b.destroy=()=>{window.__yt079Held.buffers++;destroy()};return b;};
 const complete=d.queue.onSubmittedWorkDone.bind(d.queue);d.queue.onSubmittedWorkDone=async()=>{await complete();return new Promise(resolve=>{window.__yt079Held.held=true;window.__yt079Held.release=resolve;});};return d;};
const unconfigure=GPUCanvasContext.prototype.unconfigure;GPUCanvasContext.prototype.unconfigure=function(){window.__yt079Held.unconfigured++;return unconfigure.call(this)};"#;
const HOOK: &str = "import {SceneOwner} from './scene.js'; const free=SceneOwner.prototype.free; SceneOwner.prototype.free=function(){window.__yt079Held.frees++;return free.call(this)}; window.__yt079Held.hookInstalled=true;\n";
const REFINEMENT_HOOK: &str = "import {SceneOwner} from './scene.js'; const inert=new SceneOwner(97531); window.__yt081DisposePair=[inert.dispose(),inert.dispose()]; inert.free(); window.__yt081DisposeCalls=[]; const rawDispose=SceneOwner.prototype.dispose; SceneOwner.prototype.dispose=function(){const value=rawDispose.call(this);window.__yt081DisposeCalls.push(value);return value;}; const free=SceneOwner.prototype.free; SceneOwner.prototype.free=function(){window.__yt079Held.frees++;return free.call(this)}; window.__yt079Held.hookInstalled=true;\n";
