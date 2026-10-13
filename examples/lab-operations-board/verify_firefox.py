#!/usr/bin/env python3
"""One native Firefox keyboard/Unicode/form/status journey; not OS IME or AT proof."""
from pathlib import Path
import json
import os
import re
import subprocess
import tempfile
import socket
import sys
import hashlib
from verify_local import ROOT, Client, http, port_number, wait_until, stop_owned_process, remove_owned_tree
from smoke_firefox import FirefoxDriver, W3C_ELEMENT, await_value
from smoke_forms_accessibility_firefox import keys, TAB


def run():
    out=ROOT/'target/m11-board';out.mkdir(parents=True,exist_ok=True)
    temp=Path(tempfile.mkdtemp(prefix='firefox-',dir=out));os.chmod(temp,0o700)
    port,dp=port_number(),port_number();assert port!=dp
    base=f'http://127.0.0.1:{port}'
    server=gecko=driver=client=handle=None
    report={'status':'fail','checks':[],'not_verified':['real OS IME','spoken screen reader','Safari/Edge/mobile device'],'cleanup':'pending'}
    try:
        binary=ROOT/'target/release/yetlet-lab-operations-board'
        report['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
        log=temp/'server.log';handle=log.open('x');os.chmod(log,0o600)
        server=subprocess.Popen([str(binary)],env=dict(os.environ,YETLET_BOARD_LISTEN=f'127.0.0.1:{port}',YETLET_BOARD_DB=str(temp/'board.sqlite')),stdout=handle,stderr=handle,start_new_session=True)
        wait_until(lambda:'READY ' in log.read_text(),'Board',seconds=10)
        codes={m.group(1):m.group(2) for m in re.finditer(r'^LOCAL_LOGIN (alice|bob|viewer) ([0-9a-f]{64})$',log.read_text(),re.M)}
        gecko=subprocess.Popen(['/snap/bin/geckodriver','--host','127.0.0.1','--port',str(dp)],env=dict(os.environ,TMPDIR=str(temp)),stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
        driver=FirefoxDriver(dp)
        await_value(lambda:driver.call('/status')['ready'],'geckodriver')
        report['browser']=driver.create_session()
        driver.command('/window/rect','POST',{'width':1280,'height':960})
        def js(s):return driver.evaluate(s)
        def pick(selector):
            nodes=driver.command('/elements','POST',{'using':'css selector','value':selector})
            assert len(nodes)==1
            return nodes[0][W3C_ELEMENT]
        def click(selector):
            node=pick(selector);driver.command(f'/element/{node}/click','POST',{});js('document.readyState')
        def type_text(selector,text):
            node=pick(selector);driver.command(f'/element/{node}/value','POST',{'text':text});assert js(f'document.querySelector({json.dumps(selector)}).value')==text
        def clear(selector):
            node=pick(selector);driver.command(f'/element/{node}/clear','POST',{});assert js(f'document.querySelector({json.dumps(selector)}).value')==''
        driver.navigate(base+'/board/login')
        assert http(base+'/board/login')[1]['referrer-policy']=='same-origin'
        type_text('#username','alice')
        keys(driver,TAB)
        assert js('document.activeElement.id')=='code'
        type_text('#code',codes['alice'])
        click('.board-form button[type=submit]')
        await_value(lambda:driver.command('/url')==base+'/board/projects/1/items','native login')
        assert js("document.querySelectorAll('script').length===0")
        report['checks'].append('native login, Tab between named fields, strict script-free SSR')
        click('.board-nav a[href$="/new"]')
        assert js("!document.querySelector('[aria-invalid=true]')")
        title='本機工作項目 — Unicode'
        description='第一行\n<b>Literal markup</b>'
        type_text('#title',title)
        keys(driver,TAB)
        assert js('document.activeElement.id')=='description'
        type_text('#description',description)
        assert js("document.querySelector('label[for=description]').control===document.getElementById('description')")
        click('.board-form button[type=submit]')
        await_value(lambda:driver.command('/url')==base+'/board/projects/1/items/1','create PRG')
        assert js(f'document.body.textContent.includes({json.dumps(title)})')
        assert js("document.querySelectorAll('b').length===0 && document.body.textContent.includes('<b>Literal markup</b>')")
        report['checks'].append('native Unicode input/create/read/history and plain-text escaping; not OS composition')
        click('.board-main a[href$="/edit"]')
        clear('#title')
        click('.board-form button[type=submit]')
        assert js("document.getElementById('title').getAttribute('aria-invalid')==='true'")
        assert js("document.getElementById('title').getAttribute('aria-describedby')==='title-error'")
        assert js('document.getElementById("description").value')==description
        type_text('#title','Corrected native draft')
        click('.board-form button[type=submit]')
        await_value(lambda:driver.command('/url')==base+'/board/projects/1/items/1','corrected PRG')
        report['checks'].append('native validation retains Unicode draft and exposes error relationship; correction saves')
        driver.navigate(base+'/board/projects/1/items?live=true')
        await_value(lambda:js("document.querySelector('[data-board-stream]')?.dataset.streamState==='current'"),'SSE initial snapshot')
        client=Client(port);client.login(codes)
        assert client.call('POST','/board/projects/1/items/1',client.fields('Firefox live update',2,'active'))[0]==303
        await_value(lambda:js("document.querySelector('[data-board-stream]')?.dataset.currentRevision==='3'"),'updated SSE')
        assert js("document.querySelector('[data-stream-message]').textContent.includes('Refresh authoritative records')")
        click('[data-stream-stop]')
        assert js("document.querySelector('[data-board-stream]').dataset.streamState==='paused'")
        report['checks'].append('Firefox actual persisted SSE update, authoritative refresh notice and pause')
        report['viewport']=js('({width:innerWidth,height:innerHeight,dpr:devicePixelRatio})')
        report['status']='pass'
    except Exception as error:
        report['failure_type']=type(error).__name__
        raise
    finally:
        if client:client.close()
        if driver:driver.close()
        for p in [gecko,server]:stop_owned_process(p,process_group=True)
        if handle:handle.close()
        remove_owned_tree(temp)
        for p in [port,dp]:
            with socket.socket() as s:assert s.connect_ex(('127.0.0.1',p))!=0
        report['cleanup']='owned Firefox/geckodriver/application reaped; listeners closed; synthetic database/profile/log removed'
        (out/'firefox.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report,indent=2))

if __name__=='__main__':run()
