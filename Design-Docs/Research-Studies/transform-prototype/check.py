"""Isolated fictional-world transform feasibility test. Python standard library only.
Not Atlas code, not a production authorization model. Run: python check.py.
"""
import sqlite3, json, time, statistics, platform
from dataclasses import dataclass

@dataclass(frozen=True)
class View:
    day: int = 10
    revision: int = 2
    branch: str = 'main'
    audience: str = 'public'

# Stable typed operation IDs, existing-query adapters rather than a new query language.
OPS = {'house': ('person','house','member_of'),
       'members': ('house','person','has_member'),
       'guilds': ('person','organization','guild_member'),
       'events': ('organization','event','participated'),
       'mentions': ('source','person','mentions')}

def fixture(n):
    db=sqlite3.connect(':memory:')
    db.executescript('''CREATE TABLE node(id TEXT PRIMARY KEY,kind TEXT,hidden INT);
    CREATE TABLE edge(id TEXT PRIMARY KEY,a TEXT,b TEXT,kind TEXT,lo INT,hi INT,
      rev INT,branch TEXT,source TEXT,status TEXT,hidden INT);
    CREATE INDEX outgoing ON edge(a,kind,id);''')
    nodes=[('p','person',0),('h','house',0),('q','person',0),('secret','person',1),
      ('g','organization',0),('g2','organization',0),('e','event',0),('e2','event',0),('s','source',0)]
    rows=[('1','p','h','member_of',0,None,1,'main','s','fact',0),
      ('2','h','q','has_member',0,None,1,'main','s','fact',0),
      ('3','h','secret','has_member',0,None,1,'main','s','fact',0),
      ('4','q','g','guild_member',0,20,1,'main','s','fact',0),
      ('5','q','g2','guild_member',20,None,2,'main','s','fact',0),
      ('6','secret','g2','guild_member',0,None,1,'main','s','fact',0),
      ('7','g','e','participated',0,None,1,'main','s','fact',0),
      ('8','g2','e2','participated',0,None,2,'main','s','fact',0),
      ('9','s','q','mentions',0,None,1,'main','s','candidate',0),
      ('10','q','g2','guild_member',0,None,2,'fork','s','fact',0),
      ('11','q','g','guild_member',0,20,1,'main','s','fact',0)]
    # Scale unrelated background edges; relevant frontier stays small.
    nodes.extend((f'x{i}','person',0) for i in range(n))
    rows.extend((f'x{i}',f'x{i}','h','member_of',0,None,1,'main','s','fact',0) for i in range(n))
    db.executemany('INSERT INTO node VALUES(?,?,?)',nodes)
    db.executemany('INSERT INTO edge VALUES(?,?,?,?,?,?,?,?,?,?,?)',rows); db.commit()
    db.execute('PRAGMA query_only=ON')
    return db

# Simplified fork freezes inherited revision 1; fixture has no branch withdrawals.
def select_sql(v):
    return '''SELECT edge.id,edge.a,edge.b,edge.kind,edge.source,edge.status FROM edge
      JOIN node a ON a.id=edge.a JOIN node b ON b.id=edge.b JOIN node s ON s.id=edge.source
      WHERE edge.lo<=? AND (edge.hi IS NULL OR edge.hi>?)
      AND ((edge.branch='main' AND edge.rev<=?) OR (edge.branch=? AND edge.rev<=?))
      AND (? OR (edge.hidden=0 AND a.hidden=0 AND b.hidden=0 AND s.hidden=0))''', (
      v.day,v.day,min(v.revision,1) if v.branch=='fork' else v.revision,
      v.branch,v.revision,v.audience=='author')

def query(db,v,root,relation,cap):
    sql,args=select_sql(v)
    return db.execute(sql+' AND edge.a=? AND edge.kind=? ORDER BY edge.id LIMIT ?',args+(root,relation,cap+1)).fetchall()

def run(db,v,root,steps,max_calls=12,max_edges=40,cancel=lambda:False):
    if not steps or len(steps)>4 or not 1<=max_calls<=100 or not 1<=max_edges<=1000:
        raise ValueError('budget')
    for prev,nxt in zip(steps,steps[1:]):
        if OPS[prev][1]!=OPS[nxt][0] or prev=='mentions': raise ValueError('incompatible chain')
    frontier={root}; edges={}; calls=0; truncated=False
    for step in steps:
        next_ids=set(); input_kind,output_kind,relation=OPS[step]
        for ident in sorted(frontier):
            if cancel(): return {'status':'cancelled','edges':[], 'terminal':[],'calls':calls}
            if calls==max_calls: truncated=True; break
            row=db.execute('SELECT kind FROM node WHERE id=? AND (? OR hidden=0)',(ident,v.audience=='author')).fetchone()
            if row!=(input_kind,): raise ValueError('unavailable or wrong kind')
            calls+=1
            rows=query(db,v,ident,relation,max_edges)
            if len(rows)>max_edges: truncated=True
            for e in rows[:max_edges]:
                if db.execute('SELECT kind FROM node WHERE id=?',(e[2],)).fetchone()!=(output_kind,): raise ValueError('output kind')
                if e[0] not in edges and len(edges)==max_edges: truncated=True; break
                edges[e[0]]=e
                if e[-1]=='fact': next_ids.add(e[2])
        frontier=next_ids
        if truncated: break
    return {'status':'partial' if truncated else 'complete','edges':sorted(edges.values()),'terminal':sorted(frontier) if not truncated else [],'calls':calls}

def parity(db,v):
    # Equivalent relational join, not a separate transform runtime.
    sql,args=select_sql(v)
    return [r[0] for r in db.execute('WITH visible AS ('+sql+''') SELECT DISTINCT d.b
    FROM visible a JOIN visible b ON a.b=b.a JOIN visible c ON b.b=c.a JOIN visible d ON c.b=d.a
    WHERE a.a='p' AND a.kind='member_of' AND b.kind='has_member' AND c.kind='guild_member'
    AND d.kind='participated' AND a.status='fact' AND b.status='fact' AND c.status='fact' AND d.status='fact' ORDER BY d.b''',args)]

chain=['house','members','guilds','events']; db=fixture(0); v=View(); before=db.total_changes
for ctx,expected in [(v,['e']),(View(audience='author'),['e','e2']),(View(day=20),['e2']),
                     (View(day=20,revision=1),[]),(View(branch='fork'),['e'])]:
    out=run(db,ctx,'p',chain)
    # Fork's new guild membership at rev2 can reach g2, but inherited event e2 rev2 is absent.
    assert out['terminal']==expected==parity(db,ctx)
assert run(db,v,'p',chain)==run(db,v,'p',chain)
assert len([e for e in run(db,v,'p',chain)['edges'] if e[3]=='guild_member'])==2 # distinct sourced assertions
assert not any('secret' in e for e in run(db,v,'p',chain)['edges'])
assert run(db,v,'s',['mentions'])['edges'][0][-1]=='candidate' and not run(db,v,'s',['mentions'])['terminal']
assert run(db,v,'p',chain,max_calls=2)['status']=='partial'
assert run(db,v,'p',chain,max_edges=2)['status']=='partial'
assert run(db,v,'p',chain,cancel=lambda:True)['status']=='cancelled'
for root,steps in [('h',['guilds']),('p',['house','events']),('secret',['house']),('s',['mentions','guilds'])]:
    try: run(db,v,root,steps)
    except ValueError: pass
    else: raise AssertionError('expected denial')
assert db.total_changes==before
# Note: duplicate paths converge on one event, original separate edge IDs retained.
print('fixture assertions passed')
results=[]
for n in (1000,10000,50000):
    db=fixture(n); out=run(db,v,'p',chain); assert out['terminal']==parity(db,v)
    times=[]; joins=[]
    for _ in range(15):
        t=time.perf_counter_ns();run(db,v,'p',chain);times.append((time.perf_counter_ns()-t)/1e6)
        t=time.perf_counter_ns();parity(db,v);joins.append((time.perf_counter_ns()-t)/1e6)
    results.append(dict(background=n,edges=len(out['edges']),calls=out['calls'],chain_ms=round(statistics.median(times),3),join_ms=round(statistics.median(joins),3)))
print(json.dumps(dict(python=platform.python_version(),sqlite=sqlite3.sqlite_version,platform=platform.platform(),results=results),indent=2))
