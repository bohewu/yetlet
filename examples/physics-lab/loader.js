import { createStartupShell } from './wasm-startup.mjs';
import { createFrameRateCounter } from './frame-rate.mjs';
// Application-owned page controller. First Run alone loads Wasm and starts 3D.
const root=document.getElementById('physics-lab'), canvas=document.getElementById('scene');
const el=id=>document.getElementById(id);
const assetsAvailable=el('run').dataset.available==='true';
const startupShell=createStartupShell({root,status:el('status'),indicator:el('loading')});
let startupTicket=null;
let modulePromise=null, wasm=null, sim=null, owner=null, retirement=Promise.resolve();
let cleanupPending=false;
const frameRate=createFrameRateCounter(),scenarioLabels={falling:'Falling objects',tower:'Stack impact',dominoes:'Domino chain',slope:'Slope comparison'};
let activeScenario=root.dataset.scenario,activeCount=Number(root.dataset.count);
let phase='inert', scene=1, generation=0, cameraVersion=0, selected=1, lastUI=-Infinity;
let camera={yaw:45,pitch:55,distance:28}, adapter='', submissions=0, completions=0;
const pageScope=new AbortController();
const on=(target,type,fn,options={})=>target.addEventListener(type,fn,{...options,signal:pageScope.signal});
function parameters(){const p={scenario:el('scenario').value,count:Number(el('count').value),gravity:Number(el('gravity').value),speed:Number(el('speed').value)};if(!Object.hasOwn(scenarioLabels,p.scenario)||!Number.isInteger(p.count)||p.count<1||p.count>64||!Number.isFinite(p.gravity)||p.gravity<0||p.gravity>20||![.25,.5,1,2].includes(p.speed))throw Error('Invalid bounded preset');return p;}
function state(next,message){
 if(next==='initializing'){if(startupShell.state!=='loading')startupTicket=startupShell.begin(message);}
 else if(['lost','unavailable','invalid'].includes(next))startupShell.fail(startupTicket,message);
 else if(next==='disposed'){startupShell.cancel(startupTicket,message);startupTicket=null;}
 else if(startupShell.state==='loading')startupShell.ready(startupTicket,message);
 else if(startupShell.state==='error'){startupShell.cancel(startupTicket,message);startupTicket=null;}
 phase=next;root.dataset.state=next;if(!['lost','unavailable','invalid'].includes(next))delete root.dataset.failure;el('status').textContent=message;controls();
}
function updateRate(){const value=phase==='running'?(frameRate.fps===null?'Measuring…':frameRate.fps.toFixed(1)):['paused','suspended'].includes(phase)?'Paused':phase==='inert'?'Not started':'Unavailable';el('render-rate').textContent=`${scenarioLabels[activeScenario]} · FPS (estimate): ${value} · Objects: ${activeCount}`;root.dataset.activeScenario=activeScenario;el('slope-guide').hidden=activeScenario!=='slope';}
function controls(){const active=!!owner&&!owner.retired&&owner.ready;const running=!!sim?.running(),failed=['lost','unavailable'].includes(phase);el('run').textContent=phase==='initializing'?'Preparing…':'Run';el('run').disabled=!assetsAvailable||cleanupPending||running||failed||['initializing','invalid'].includes(phase)||(active&&(!sim?.valid()||!drawable(owner)));el('pause').disabled=!running&&!owner?.startRequested;el('step').disabled=!active||running||!sim?.valid()||!drawable(owner);el('retry').hidden=!failed;el('retry').disabled=cleanupPending||!failed||(!!sim&&!sim.valid());el('reset').disabled=cleanupPending||phase==='initializing';updateRate();}
const failureGuidance={
 'insecure-context':'3D needs a secure browser context. Open the server through its localhost/loopback address or trusted HTTPS, then choose Retry.',
 'webgpu-missing':'WebGPU is not available in this browser context. Check Browser help and your browser’s graphics diagnostics before choosing Retry.',
 'program-unavailable':'The 3D program could not be loaded or started. Check the server’s 3D files and browser site permissions, then choose Retry.',
 'adapter-unavailable':'The browser could not provide a usable 3D adapter. Check its graphics diagnostics and platform support, then choose Retry.',
 'device-unavailable':'The browser could not create a 3D device. Check its graphics diagnostics, then choose Retry.',
 'pipeline-unavailable':'The browser could not prepare this 3D renderer. Check Browser help and graphics diagnostics before choosing Retry.',
 'canvas-unavailable':'The browser could not prepare this 3D canvas. Check Browser help before choosing Retry.',
 'surface-unavailable':'The browser could not prepare this 3D canvas. Check Browser help before choosing Retry.',
 'surface-error':'The 3D canvas stopped drawing. Check Browser help, then choose Retry to restore it.',
 'device-lost':'The browser lost the 3D device. The simulation is paused. Choose Retry to restore 3D; it will remain paused.',
 'resource-exhausted':'The browser could not supply 3D resources. Check its graphics diagnostics before choosing Retry.',
 'validation-error':'The browser rejected a 3D rendering operation. Choose Retry; if it repeats, report this category with your browser diagnostics.',
 'buffer-budget':'The 3D resource checks failed. Check Browser help and report this category if Retry fails again.',
 'initialization-timeout':'3D preparation timed out. Retry stays disabled until cleanup finishes. If it stays disabled, reload this page; reload discards the tab-local simulation.',
 'invalid-preset':'The preset values are outside the displayed bounds. Correct them and choose Reset before starting again.',
 'invalid-simulation':'The simulation stopped because its state is invalid. The last valid snapshot is retained. Choose Reset before running again.',
 'unexpected':'3D stopped unexpectedly. Check Browser help before choosing Retry.'
};
function showFailure(next,error){let code=String(error?.message??error);if(code==='Invalid bounded preset')code='invalid-preset';if(!Object.hasOwn(failureGuidance,code))code=root.dataset.initializationStage==='module'?'program-unavailable':'unexpected';root.dataset.failure=code;state(next,failureGuidance[code]+' The scene table remains readable.');el('browser-help').open=true;}
function rows(){return sim?sim.poses():new Float32Array();}
function updateUI(force=false,now=performance.now()){
 if(!sim||(!force&&now-lastUI<250))return;lastUI=now;const poses=rows();const tbody=el('objects');
 if(tbody.children.length!==poses.length/19-5){tbody.replaceChildren();for(let i=0;i<poses.length;i+=19){if(poses[i]>=1000)continue;const tr=document.createElement('tr');tr.dataset.object=String(poses[i]);for(let c=0;c<4;c++)tr.append(document.createElement('td'));tbody.append(tr);}el('selection').replaceChildren();for(const tr of tbody.children){const o=document.createElement('option');o.value=tr.dataset.object;o.textContent=`Object ${o.value}`;el('selection').append(o);}}
 let detail='Select an object.';for(let i=0;i<poses.length;i+=19){const id=poses[i];if(id>=1000)continue;const tr=tbody.querySelector(`[data-object="${id}"]`);if(!tr)continue;
 const pos=Array.from(poses.slice(i+2,i+5)).map(x=>x.toFixed(2)).join(', '),vel=Array.from(poses.slice(i+16,i+19)).map(x=>x.toFixed(2)).join(', ');
 [String(id),poses[i+1]===1?'Sphere':'Box',pos,vel].forEach((value,c)=>tr.children[c].textContent=value);tr.dataset.selected=String(id===selected);if(id===selected)detail=`Object ${id} · position (${pos}) m · velocity (${vel}) m/s`;
 }
 el('selection').value=String(selected);el('detail').textContent=detail;
 el('metrics').textContent=`Ticks: ${sim.ticks()} · simulated: ${(sim.ticks()/60).toFixed(3)} s · scheduler-discarded: ${sim.dropped_seconds().toFixed(3)} s`;
 updateRate();
 el('objects').closest('table').querySelector('caption').textContent=sim.valid()?'Current valid simulation snapshot (tab-local).':'Last valid snapshot; Reset required.';
}
function stop(o=owner){frameRate.reset();sim?.pause();if(o)o.startRequested=false;if(o?.raf){cancelAnimationFrame(o.raf);o.raf=0;}controls();}
async function retire(o){if(!o||o.retired)return;cleanupPending=true;controls();o.retired=true;if(o.timer)clearTimeout(o.timer);if(o.raf)cancelAnimationFrame(o.raf);o.raf=0;o.scope.abort();o.resize?.disconnect();o.intersection?.disconnect();o.gpu?.dispose();try{await o.initialization;}catch{}if(o.gpu&&!o.freed){o.gpu.free();o.freed=true;}cleanupPending=false;controls();}
function fail(o,category){if(owner!==o||o.retired)return;stop(o);owner=null;retirement=retire(o);showFailure(sim?.valid()?'lost':'invalid',sim?.valid()?category:'invalid-simulation');updateUI(true);}
window.physicsLabFault=(gen,category)=>queueMicrotask(()=>{if(owner?.generation===gen)fail(owner,category);});
window.physicsLabProgress=(gen,stage)=>queueMicrotask(()=>{if(owner?.generation===gen&&!owner.retired){root.dataset.initializationStage=stage;el('status').textContent=`Preparing 3D…${owner.startRequested?'':' It will remain paused when ready.'}`;}});
window.physicsLabCompletion=gen=>queueMicrotask(()=>{if(owner?.generation===gen&&!owner.retired)completions++;});
function visibleCanvas(o){if(!o||owner!==o||o.retired||document.visibilityState!=='visible'||!o.inView||!canvas.isConnected)return false;const r=canvas.getBoundingClientRect();return r.width>0&&r.height>0&&r.bottom>0&&r.right>0&&r.top<innerHeight&&r.left<innerWidth;}
function drawable(o){return !!o?.ready&&visibleCanvas(o);}
function draw(o){
 if(!drawable(o))return false;
 const rect=canvas.getBoundingClientRect();if(!o.gpu.resize(rect.width,rect.height,devicePixelRatio,canvas.isConnected)){suspend(o);return false;}
 o.gpu.draw(rows(),camera.yaw,camera.pitch,camera.distance);o.drawn={scene,ticks:sim.ticks(),camera:cameraVersion};submissions++;if(sim?.running())frameRate.record(performance.now());return true;
}
function frame(o,now){o.raf=0;if(!drawable(o)){suspend(o);return;}
 try{if(sim.running())sim.advance(now);if(!sim.valid())throw Error('invalid-simulation');
  if(!sim.running()||now-o.lastDraw>=1000/60){if(draw(o))o.lastDraw=now;}updateUI(false,now);
  if(sim.running())queue(o);
 }catch(error){if(!sim.valid()){stop(o);showFailure('invalid','invalid-simulation');updateUI(true);}else fail(o,error);}}
function queue(o=owner){if(o&&!o.retired&&!o.raf)o.raf=requestAnimationFrame(()=>frame(o,performance.now()));}
function suspend(o){if(owner!==o||o.retired)return;stop(o);if(o.ready&&phase!=='invalid')state('suspended','Simulation paused while the canvas is hidden. Return, then press Run.');}
function attach(o){
 o.resize=new ResizeObserver(()=>{if(owner!==o)return;cameraVersion++;if(!visibleCanvas(o))suspend(o);else if(o.ready)queue(o);});o.resize.observe(canvas);
 o.intersection=new IntersectionObserver(entries=>{if(owner!==o)return;o.inView=entries[0]?.isIntersecting===true;if(!o.inView)suspend(o);else if(phase==='suspended')state('paused','Canvas visible. Press Run to continue.');});o.intersection.observe(canvas);
 document.addEventListener('visibilitychange',()=>{if(owner!==o)return;if(document.visibilityState!=='visible')suspend(o);else if(phase==='suspended')state('paused','Tab visible. Press Run to continue.');},{signal:o.scope.signal});
}
function start(o){if(!drawable(o)||!sim?.valid())return;sim.resume(performance.now());frameRate.reset(performance.now());o.startRequested=false;state('running','Running. Pause to inspect an object; Reset rebuilds the current preset.');queue(o);}
async function initialize(retry=false,startOnReady=false){
 if(['initializing','running','paused','suspended'].includes(phase))return;
 if(cleanupPending || (retry&&sim&&!sim.valid()))return;
 const o={generation:++generation,startRequested:startOnReady,scope:new AbortController(),gpu:null,initialization:null,freed:false,retired:false,ready:false,raf:0,lastDraw:-Infinity,inView:true,drawn:null};owner=o;
 state('initializing','Preparing 3D…');root.dataset.initializationStage='module';attach(o);
 try{
  await retirement;if(owner!==o||o.retired)return;
  if(!isSecureContext)throw Error('insecure-context');
  if(!navigator.gpu)throw Error('webgpu-missing');
  if(!modulePromise)modulePromise=import('./lab.js').then(async m=>{await m.default();return m;}).catch(e=>{modulePromise=null;throw e;});
  wasm=await modulePromise;if(owner!==o||o.retired)return;
  root.dataset.initializationStage="physics";
  if(!sim){const p=parameters();sim=wasm.Simulation.new(p.count,p.gravity,p.speed,p.scenario);activeScenario=p.scenario;activeCount=p.count;selected=1;}
  o.gpu=new wasm.Gpu(o.generation);o.initialization=o.gpu.initialize(canvas);
  try{adapter=await Promise.race([o.initialization,new Promise((_,reject)=>{o.timer=setTimeout(()=>reject(Error('initialization-timeout')),15000);o.scope.signal.addEventListener("abort",()=>{clearTimeout(o.timer);reject(Error("retired-owner"));},{once:true});})]);}finally{clearTimeout(o.timer);o.timer=0;}if(owner!==o||o.retired)return;
  o.ready=true;state('paused','Ready. Press Run to start, or inspect the paused scene.');if(o.startRequested&&visibleCanvas(o))start(o);else if(!visibleCanvas(o))suspend(o);else queue(o);updateUI(true);
 }catch(error){if(owner===o&&!o.retired){stop(o);owner=null;retirement=retire(o);showFailure(sim?'lost':'unavailable',error);}}
}
async function dispose(){const o=owner;owner=null;stop(o);retirement=retire(o);if(sim){sim.free();sim=null;}state('disposed','3D stopped. Press Run to prepare a new world.');await retirement;}
function guard(fn){return()=>{try{fn();}catch(error){el('status').textContent=String(error);}};}
on(el('retry'),'click',()=>initialize(true));
on(el('run'),'click',()=>{if(cleanupPending||['initializing','running','invalid','lost','unavailable'].includes(phase))return;if(!owner)return initialize(false,true);start(owner);});
on(el('pause'),'click',guard(()=>{if(!sim?.running()&&!owner?.startRequested)return;stop();if(phase==='initializing'){state('initializing','Preparing 3D; it will remain paused when ready.');return;}state('paused','Paused. Press Run to continue, or Reset to rebuild the preset.');queue();updateUI(true);}));
on(el('step'),'click',guard(()=>{if(!drawable(owner)||sim.running())return;sim.step();state('paused','Advanced exactly one physics step; still paused.');queue();updateUI(true);}));
on(el('parameters'),'submit',event=>{if(cleanupPending||phase==='initializing'){event.preventDefault();el('status').textContent='Wait for 3D preparation to finish';return;}if(!sim)return;event.preventDefault();try{if(cleanupPending||phase==='initializing')throw Error('Wait for 3D preparation to finish');const p=parameters();stop();if(owner?.ready)state('paused','Resetting the current preset…');sim.reset(p.count,p.gravity,p.speed,p.scenario);activeScenario=p.scenario;activeCount=p.count;scene++;selected=1;if(owner?.ready){state('paused','Preset reset. Press Run to start.');queue();}else state('lost','Preset reset. Press Retry to restore 3D.');updateUI(true);}catch(error){el('status').textContent=String(error);}});
on(el('selection'),'change',()=>{selected=Number(el('selection').value);updateUI(true);});
function cameraChanged(){const next={yaw:Number(el('yaw').value),pitch:Number(el('pitch').value),distance:Number(el('distance').value)};if(!Number.isFinite(next.yaw)||next.yaw< -180||next.yaw>180||!Number.isFinite(next.pitch)||next.pitch<10||next.pitch>80||!Number.isFinite(next.distance)||next.distance<8||next.distance>70){el('status').textContent='Camera values outside bounds.';return;}camera=next;cameraVersion++;queue();}
for(const id of ['yaw','pitch','distance'])on(el(id),'change',cameraChanged);
on(el('camera-reset'),'click',()=>{el('yaw').value='45';el('pitch').value='55';el('distance').value='28';cameraChanged();});
let drag=null;
on(canvas,'pointerdown',e=>{if(!owner?.ready)return;drag={x:e.clientX,y:e.clientY,moved:false};canvas.setPointerCapture(e.pointerId);});
on(canvas,'pointermove',e=>{if(!drag||!owner?.ready)return;const dx=e.clientX-drag.x,dy=e.clientY-drag.y;if(Math.abs(dx)+Math.abs(dy)>2)drag.moved=true;if(drag.moved){el('yaw').value=String(Math.max(-180,Math.min(180,camera.yaw+dx*.4)));el('pitch').value=String(Math.max(10,Math.min(80,camera.pitch+dy*.4)));cameraChanged();}drag.x=e.clientX;drag.y=e.clientY;});
on(canvas,'pointerup',e=>{const moved=drag?.moved;drag=null;if(moved||!owner?.ready||sim?.running()||!sim?.valid())return;const o=owner;if(o.drawn?.scene!==scene||o.drawn.ticks!==sim.ticks()||o.drawn.camera!==cameraVersion){queue();el('status').textContent='View updated; click again to select.';return;}try{const r=canvas.getBoundingClientRect();const ray=o.gpu.ray((e.clientX-r.left)/r.width,(e.clientY-r.top)/r.height,camera.yaw,camera.pitch,camera.distance);const id=sim.pick(...ray);if(id){selected=id;updateUI(true);}}catch(error){fail(o,String(error));}});
on(canvas,'wheel',e=>{if(!owner?.ready)return;e.preventDefault();el('distance').value=String(Math.max(8,Math.min(70,camera.distance+Math.sign(e.deltaY))));cameraChanged();},{passive:false});
on(window,'pagehide',()=>{dispose();startupShell.dispose();pageScope.abort();});
el('pitch').value='55';controls();
// Local diagnostics for focused product verification; no framework API or auto retry.
window.physicsLabDiagnostics={snapshot:()=>({phase,scene,generation,activeScenario,activeCount,fps:frameRate.fps,submissions,completions,adapter,cleanupPending,initializationStage:root.dataset.initializationStage,ticks:sim?.ticks()??0,dropped:sim?.dropped_seconds()??0,valid:sim?.valid()??false,poses:sim?Array.from(rows()):[],rafPending:!!owner?.raf,drawn:owner?.drawn??null}),lose:()=>owner?.gpu?.lose(),fault:(gen,category)=>window.physicsLabFault(gen,category),dispose};
