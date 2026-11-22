"""Research #66: isolated constraint/unknown and measured label experiment.
Run beside the original prototype.html: python followup.py
Requires Shapely and Playwright with system Chromium. No application DB.
"""
from pathlib import Path
import hashlib, json, random, statistics, time, platform
import shapely
from shapely.geometry import Point, Polygon, box
from shapely.ops import unary_union
from playwright.sync_api import sync_playwright

HERE = Path(__file__).parent
EXTENT = box(0, 0, 100, 80)
LOCK = box(0, 0, 20, 80)
RECORDS = [
    {'id':'locked', 'anchor':[10,40], 'geometry':[list(p) for p in LOCK.exterior.coords]},
    {'id':'east', 'anchor':[75,40], 'geometry':None},
    {'id':'proposal', 'anchor':None, 'geometry':None, 'allowed_box':[30,15,55,65]},
    {'id':'unknown', 'anchor':None, 'geometry':None},
]

def generate(records, seed):
    # Only one locked polygon/convex extent in this deliberately bounded spike.
    records = sorted(records, key=lambda r:r['id'])
    fixed = next(r for r in records if r['id']=='locked')
    locked = Polygon(fixed['geometry'])
    free = EXTENT.difference(locked)
    errors, unknown, anchors = [], [], []
    for r in records:
        if r['id']=='locked': continue
        a = r['anchor']
        if a is None and 'allowed_box' in r:
            rng = random.Random(int.from_bytes(hashlib.sha256(f'v1:{seed}:{r["id"]}'.encode()).digest()[:8], 'big'))
            x,y,X,Y = r['allowed_box']
            a = [rng.uniform(x,X),rng.uniform(y,Y)]
        if a is None:
            unknown.append(r['id']); continue
        if not free.contains(Point(a)) or (r.get('allowed_box') and not box(*r['allowed_box']).covers(Point(a))):
            errors.append(r['id']+': anchor outside permitted free region')
        if any(a==b for _,b in anchors): errors.append(r['id']+': duplicate anchor')
        anchors.append((r['id'],a))
    if errors: return {'errors':errors,'unknown':unknown,'draft':{}}
    # Bound Voronoi cells by clipping the rectangular extent against each bisector.
    def clip(poly,a,b):
        def d(p): return 2*(b[0]-a[0])*p[0]+2*(b[1]-a[1])*p[1]+sum(x*x for x in a)-sum(x*x for x in b)
        out=[]
        for p,q in zip(poly,poly[1:]+poly[:1]):
            u,v=d(p),d(q)
            if u<=0: out.append(p)
            if (u<0<v) or (v<0<u):
                f=u/(u-v); out.append([p[0]+f*(q[0]-p[0]),p[1]+f*(q[1]-p[1])])
        return out
    draft={}
    for name,a in anchors:
        poly=[[0,0],[100,0],[100,80],[0,80]]
        for other,b in anchors:
            if name!=other: poly=clip(poly,a,b)
        g=shapely.set_precision(Polygon(poly).intersection(free),1e-8)
        if not g.is_valid or g.is_empty or g.geom_type!='Polygon' or not g.covers(Point(a)):
            return {'errors':[name+': invalid candidate'],'unknown':unknown,'draft':{}}
        draft[name]={'anchor':a,'geometry':list(g.exterior.coords),'status':'draft'}
    return {'errors':[],'unknown':unknown,'draft':draft,'locked':fixed['geometry']}

original=json.dumps(RECORDS,sort_keys=True)
a=generate(RECORDS,66); b=generate(RECORDS,67)
assert not a['errors'] and not b['errors']
assert a==generate(list(reversed(RECORDS)),66)==generate(json.loads(original),66)
extra=generate(RECORDS+[{'id':'extra-unknown','anchor':None,'geometry':None}],66)
assert extra['draft']==a['draft'] and extra['unknown']==['extra-unknown','unknown']
assert a['unknown']==['unknown'] and 'unknown' not in a['draft']
assert a['locked']==b['locked']==RECORDS[0]['geometry']
assert a['draft']['east']['anchor']==b['draft']['east']['anchor']==[75,40]
assert a['draft']['proposal']['anchor']!=b['draft']['proposal']['anchor']
assert json.dumps(RECORDS,sort_keys=True)==original
for result in (a,b):
    parts=[Polygon(d['geometry']) for d in result['draft'].values()]+[LOCK]
    assert all(p.is_valid for p in parts)
    assert abs(sum(p.area for p in parts)-unary_union(parts).area)<1e-5
    assert unary_union(parts).symmetric_difference(EXTENT).area<1e-5
    assert parts[0].boundary.intersection(parts[1].boundary).length>0
bad=[dict(r,anchor=[10,10]) if r['id']=='east' else r for r in RECORDS]
assert generate(bad,66)['errors'] and not generate(bad,66)['draft']
print('constraint and unknown assertions passed')
with sync_playwright() as pw:
    browser=pw.chromium.launch(executable_path='/usr/bin/chromium',args=['--no-sandbox','--disable-dev-shm-usage'])
    page=browser.new_page(viewport={'width':800,'height':630},device_scale_factor=1)
    page.set_content((HERE/'prototype.html').read_text())
    page.wait_for_selector('svg')
    metrics=page.evaluate('''()=>{
      const overlap=(a,b)=>a.x<b.x+b.width&&a.x+a.width>b.x&&a.y<b.y+b.height&&a.y+a.height>b.y;
      const pairs=bs=>bs.reduce((n,a,i)=>n+bs.slice(i+1).filter(b=>overlap(a,b)).length,0);
      const map=document.getElementById('map');
      const out=[];
      for(const n of [25,100,250]){
        const fs=Array.from({length:n},(_,i)=>({kind:'point',id:'p'+String(i).padStart(3,'0'),x:35+(i*53%560),y:35+(i*31%285),label:'Northbridge '+i}));
        map.innerHTML=lab.svg(fs);
        const current=[...map.querySelectorAll('text')].map(t=>{let b=t.getBBox();return {x:b.x-1.5,y:b.y-1.5,width:b.width+3,height:b.height+3}});
        const svg=map.querySelector('svg'); svg.querySelectorAll('text').forEach(x=>x.remove());
        const boxes=[],start=performance.now();
        for(const f of fs){
          const t=document.createElementNS(svg.namespaceURI,'text');t.textContent=f.label;t.setAttribute('font-family','sans-serif');t.setAttribute('font-size','12');t.setAttribute('fill','#273c39');t.setAttribute('stroke','#f2eee3');t.setAttribute('stroke-width','3');t.setAttribute('paint-order','stroke');svg.append(t);
          const w=t.getBBox().width;let accepted=false;
          for(const [dx,dy] of [[8,-8],[8,18],[-w-8,-8],[-w-8,18]]){
            t.setAttribute('x',f.x+dx);t.setAttribute('y',f.y+dy);let b=t.getBBox(),r={x:b.x-1.5,y:b.y-1.5,width:b.width+3,height:b.height+3};
            if(r.x>=0&&r.y>=0&&r.x+r.width<=640&&r.y+r.height<=360&&!boxes.some(b=>overlap(r,b))){boxes.push(r);accepted=true;break;}
          }
          if(!accepted)t.remove();
        }
        out.push({n,baselinePlaced:current.length,baselinePairs:pairs(current),measuredPlaced:boxes.length,measuredPairs:pairs(boxes),placementMs:performance.now()-start});
      }
      return out;
    }''')
    assert all(m['measuredPairs']==0 and 0<m['measuredPlaced']<=m['n'] for m in metrics)
    # Render actual output from the constraint experiment, not a hand-drawn mock.
    page.evaluate('''data=>{const m=document.getElementById('map'),scale=([x,y])=>[35+x*5.7,35+y*3.625];let fs=[lab.P('locked',[data.locked.map(scale)],'#a9c9a5'),...Object.entries(data.draft).map(([id,r])=>lab.P(id,[r.geometry.map(scale)],'#d7c2e5',{draft:true})),...Object.entries(data.draft).map(([id,r])=>{let [x,y]=scale(r.anchor);return {id,kind:'point',x,y,label:id+' (draft cell)'}})];m.innerHTML=lab.svg(fs);document.getElementById('legend').textContent='CONSTRAINT RESEARCH · locked region + draft cells';document.getElementById('detail').textContent='Unplaced (no coordinates): '+data.unknown.join(', '); }''',a)
    page.screenshot(path=str(HERE/'followup.png'))
    print(json.dumps({'browser':browser.version,'platform':platform.platform(),'shapely':shapely.__version__,'labels':metrics,'unknown':a['unknown'],'draft_ids':list(a['draft']),'negative':generate(bad,66)['errors']},indent=2))
    browser.close()
