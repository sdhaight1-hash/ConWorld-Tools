from playwright.sync_api import sync_playwright
from shapely.geometry import Polygon, Point
from shapely.ops import unary_union
from pathlib import Path
import json,platform,shapely
p=Path(__file__).parent
with sync_playwright() as pw:
 b=pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
 page=b.new_page(viewport={'width':800,'height':530},device_scale_factor=1)
 page.set_content((p/'prototype.html').read_text());page.wait_for_selector('svg')
 data=page.evaluate('''()=>{let {region,candidate,cells,scene,svg,labels}=lab;let a=region(99,'main',false),b=region(100,'main',false);return {a,b,candidate:candidate(),same:JSON.stringify(candidate())===JSON.stringify(candidate()),privateSame:svg(region(99,'main',false,true))===svg(region(99,'main',false,false)),authorDifferent:svg(region(99,'main',true,true))!==svg(region(99,'main',true,false)),fork:svg(region(99,'main',false))===svg(region(100,'fork',false)),lock:candidate(1).find(x=>x.id==='anchor-0'),collision:labels([lab.region(99,'main',false)[5],{kind:'point',id:'x',x:180,y:170,label:'Another place'}]).map(x=>x.at)}}''')
 assert data['same'] and data['privateSame'] and data['authorDifferent'] and data['fork'];assert data['lock']['x']==180 and data['lock']['y']==170
 a={f['id']:f for f in data['a']};e={f['id']:f for f in data['b']};assert a['east']['rings']==e['east']['rings'] and a['east']['fill']!=e['east']['fill']
 west=Polygon(a['west']['rings'][0],a['west']['rings'][1:]);east=Polygon(a['east']['rings'][0]);assert west.is_valid and east.is_valid and west.intersection(east).area<1e-6 and west.boundary.intersection(east.boundary).length>0;assert not west.contains(Point(120,160))
 polys=[Polygon(f['rings'][0]) for f in data['candidate'] if f['kind']=='polygon'];assert all(x.is_valid for x in polys);assert abs(sum(x.area for x in polys)-unary_union(polys).area)<1e-6;assert abs(unary_union(polys).area-570*290)<1e-6
 page.locator('[data-id=town]').click();assert page.locator('#mode').input_value()=='town';page.locator('[data-id=manor]').click();assert page.locator('#mode').input_value()=='manor';page.locator('#mode').select_option('ship');s=page.locator('svg').text_content();page.locator('#day').select_option('100');assert s!=page.locator('svg').text_content();page.locator('#mode').select_option('schematic');assert 'not to scale' in page.locator('#legend').inner_text()
 page.locator('#mode').select_option('regional');page.screenshot(path=str(p/'regional.png'));page.locator('#mode').select_option('candidate');page.screenshot(path=str(p/'candidate.png'))
 stats=page.evaluate('''()=>{let out=[],canvas=document.createElement('canvas');canvas.width=640;canvas.height=360;let ctx=canvas.getContext('2d');let host=document.createElement('div');host.style.cssText='position:absolute;left:0;top:0;visibility:hidden';document.body.append(host);for(let n of [1000,10000,50000]){let f=Array.from({length:n},(_,i)=>lab.P('f'+i,[lab.box(i%250*2,Math.floor(i/250)*1.5,1.5,1)],'#c8ad80'));let rows=[];for(let j=0;j<5;j++){let a=performance.now(),s=lab.svg(f),b=performance.now();host.innerHTML=s;host.getBoundingClientRect();let c=performance.now();lab.canvas(f,ctx);ctx.getImageData(0,0,1,1);let d=performance.now();rows.push([b-a,c-b,d-c,new TextEncoder().encode(s).length]);host.replaceChildren();}out.push({n,median:rows[0].map((_,i)=>rows.map(r=>r[i]).sort((a,b)=>a-b)[2])});}return out}''')
 print(json.dumps({'browser':b.version,'platform':platform.platform(),'shapely':shapely.__version__,'valid':True,'benchmark':stats},indent=2));b.close()
