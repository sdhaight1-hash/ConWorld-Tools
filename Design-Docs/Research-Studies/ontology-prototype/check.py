"""Research #71: independent synthetic comparison, not production Atlas code."""
import sqlite3, time, statistics, json, platform
from dataclasses import dataclass

@dataclass(frozen=True)
class View:
    day: int = 30
    rev: int = 3
    branch: str = 'main'
    author: bool = False
    world: str = 'vale'

# Explicit allowlisted contracts; no inferred semantic links from classification.
RULES = {'holds': ('person','title'), 'governs': ('org','place'),
 'adheres': ('person','concept'), 'located': ('org','place'),
 'uses': ('org','concept'), 'produces': ('event','asset'),
 'material': ('asset','concept'), 'tool': ('event','asset'),
 'authored': ('person','concept'), 'embodies': ('asset','concept'),
 'speaks': ('person','concept'), 'species': ('person','concept'),
 'signatory': ('org','concept'), 'obligation': ('concept','org'),
 'tenure': ('household','place'), 'works_at': ('person','org'),
 'market': ('place','org'), 'descended_culture': ('concept','concept')}
CLASSES = {'faith':'concept','language':'concept','culture':'concept',
 'material':'concept','work':'concept','species':'concept','agreement':'concept',
 'technology':'concept','currency':'concept','market':'org','guild':'org',
 'town':'place','manor':'place','artifact':'asset','manuscript':'asset',
 'occupation':'concept','settlement':'place','secret_guild':'org'}
# Two classification models, same explicit multi-class semantics: indexed rows vs JSON arrays.
FIX = [('king','person'),('queen','person'),('crown','title'),('court','org'),
 ('vale','place'),('faith','concept'),('temple','org'),('town','place'),
 ('miner','person'),('guild','org'),('ore','concept'),('sword','asset'),
 ('forge','event'),('hammer','asset'),('work','concept'),('copy','asset'),
 ('scribe','person'),('speech','concept'),('culture','concept'),('islander','person'),
 ('species','concept'),('automaton','person'),('treaty','concept'),('league','org'),
 ('farm','place'),('household','household'),('market','org'),('tech','concept'),
 ('oldculture','concept'),('secret','org'),('records','source'),('secret_source','source')]
FACTS = [('king','holds','crown',0,20),('queen','holds','crown',20,None),
 ('court','governs','vale',0,None),('king','adheres','faith',0,None),
 ('temple','located','town',0,20),('guild','uses','tech',0,None),
 ('forge','produces','sword',10,None),('sword','material','ore',10,None),
 ('forge','tool','hammer',10,None),('scribe','authored','work',0,None),
 ('copy','embodies','work',0,None),('islander','speaks','speech',0,None),
 ('automaton','species','species',0,None),('court','signatory','treaty',0,None),
 ('league','signatory','treaty',0,None),('guild','signatory','treaty',0,None),
 ('treaty','obligation','guild',20,None),('household','tenure','farm',0,None),
 ('miner','works_at','guild',0,None),('town','market','market',10,None),
 ('culture','descended_culture','oldculture',20,None)]
TAGS = [('faith','faith'),('speech','language'),('culture','culture'),('oldculture','culture'),
 ('ore','material'),('work','work'),('species','species'),('treaty','agreement'),
 ('tech','technology'),('guild','guild'),('town','town'),('town','settlement'),
 ('farm','manor'),('sword','artifact'),('copy','manuscript'),('market','market'),('secret','secret_guild')]

def build(n=0):
    db=sqlite3.connect(':memory:'); db.execute('PRAGMA foreign_keys=ON')
    db.executescript('''CREATE TABLE obj(w TEXT,id TEXT,kind TEXT,label TEXT,hidden INT NOT NULL DEFAULT 0,PRIMARY KEY(w,id));
    CREATE TABLE cls(id TEXT PRIMARY KEY,kind TEXT NOT NULL);
    CREATE TABLE f(w TEXT,id TEXT,a TEXT,p TEXT,b TEXT,src TEXT,lo INT,hi INT,rev INT,branch TEXT,status TEXT,hidden INT,PRIMARY KEY(w,id),FOREIGN KEY(w,a) REFERENCES obj(w,id),FOREIGN KEY(w,b) REFERENCES obj(w,id),FOREIGN KEY(w,src) REFERENCES obj(w,id),CHECK(hi IS NULL OR lo<hi));
    CREATE INDEX fa ON f(w,a,p,id);
    CREATE TABLE c(w TEXT,id TEXT,tag TEXT,src TEXT,lo INT,hi INT,rev INT,branch TEXT,PRIMARY KEY(w,id,tag,rev,branch),FOREIGN KEY(w,id) REFERENCES obj(w,id),FOREIGN KEY(tag) REFERENCES cls(id),FOREIGN KEY(w,src) REFERENCES obj(w,id));
    CREATE INDEX ct ON c(w,tag,id);
    CREATE TABLE cj(w TEXT,id TEXT,doc TEXT CHECK(json_valid(doc)),PRIMARY KEY(w,id),FOREIGN KEY(w,id) REFERENCES obj(w,id));
    CREATE TABLE val(w TEXT,id TEXT,field TEXT,amount REAL,unit TEXT,state TEXT,PRIMARY KEY(w,id,field),FOREIGN KEY(w,id) REFERENCES obj(w,id),CHECK((state='unknown' AND amount IS NULL) OR (state='known' AND amount>=0 AND amount IS NOT NULL)));''')
    db.executemany('INSERT INTO cls VALUES(?,?)',CLASSES.items())
    db.executemany('INSERT INTO obj VALUES(?,?,?,?,?)',[('vale',i,k,i,int(i in ('secret','secret_source'))) for i,k in FIX]+[('other','king','person','king',0)])
    def fact(id,a,p,b,lo=0,hi=None,rev=1,branch='main',status='fact',src='records',w='vale'):
        def kind(x):
            row=db.execute('SELECT kind FROM obj WHERE w=? AND id=?',(w,x)).fetchone()
            return row[0] if row else None
        if (kind(a),kind(b))!=RULES[p] or kind(src)!='source': raise ValueError('endpoint or world')
        db.execute('INSERT INTO f VALUES(?,?,?,?,?,?,?,?,?,?,?,0)',(w,id,a,p,b,src,lo,hi,rev,branch,status))
    for ix,(a,p,b,lo,hi) in enumerate(FACTS): fact(str(ix),a,p,b,lo,hi)
    fact('fork','king','holds','crown',20,40,2,'fork',status='claim')
    fact('later','king','speaks','speech',0,None,3)
    fact('secret','miner','works_at','secret',0,None,1)
    fact('src_hidden','queen','adheres','faith',0,None,1,src='secret_source')
    for args in [('bad','town','holds','crown'),('bad','king','holds','crown',0,None,1,'main','fact','records','other')]:
        try: fact(*args)
        except ValueError: pass
        else: raise AssertionError('wrong kind/world')
    def tag(i,c,lo=0,hi=None,rev=1,branch='main',src='records'):
        if db.execute('SELECT kind FROM obj WHERE w="vale" AND id=?',(i,)).fetchone()!=(CLASSES[c],): raise ValueError('classification kind')
        db.execute('INSERT INTO c VALUES("vale",?,?,?,?,?,?,?)',(i,c,src,lo,hi,rev,branch))
    for i,c in TAGS: tag(i,c)
    tag('town','manor',20,None,2,'fork'); tag('farm','settlement',25,None,3)
    tag('guild','secret_guild',src='secret_source')
    try: tag('king','town')
    except ValueError: pass
    else: raise AssertionError('class kind')
    # Dense matching cohort, same background for both stores.
    for i in range(n):
        ident=f'x{i:05d}';db.execute('INSERT INTO obj VALUES("vale",?,"org",?,0)',(ident,ident));tag(ident,'guild' if i%2==0 else 'market')
    for (ident,) in db.execute('SELECT id FROM obj WHERE w="vale"').fetchall():
        rows=db.execute('SELECT tag,src,lo,hi,rev,branch FROM c WHERE w="vale" AND id=? ORDER BY tag,rev,branch',(ident,)).fetchall()
        db.execute('INSERT INTO cj VALUES("vale",?,?)',(ident,json.dumps([dict(zip(('tag','src','lo','hi','rev','branch'),x)) for x in rows],separators=(',',':'))))
    db.execute('INSERT INTO val VALUES("vale","farm","area",12,"ha","known")')
    db.execute('INSERT INTO val VALUES("vale","town","area",NULL,"ha","unknown")')
    try: db.execute('INSERT INTO val VALUES("vale","town","mass",-1,"kg","known")')
    except sqlite3.IntegrityError: pass
    else: raise AssertionError('negative')
    db.commit();return db

def args(v):return (v.world,v.day,v.day,min(v.rev,2) if v.branch=='fork' else v.rev,v.branch,v.rev,int(v.author))
def facts(db,v,a,p,cap=100):
    sql='''SELECT f.id,f.b,f.src,f.status FROM f JOIN obj a ON a.w=f.w AND a.id=f.a JOIN obj b ON b.w=f.w AND b.id=f.b JOIN obj s ON s.w=f.w AND s.id=f.src
      WHERE f.w=? AND f.lo<=? AND (f.hi IS NULL OR f.hi>?) AND ((f.branch='main' AND f.rev<=?) OR (f.branch=? AND f.rev<=?)) AND (? OR (a.hidden=0 AND b.hidden=0 AND s.hidden=0 AND f.hidden=0)) AND f.a=? AND f.p=? ORDER BY f.id LIMIT ?'''
    return db.execute(sql,args(v)+(a,p,cap+1)).fetchall()
def classes(db,v,tag,model,cap=100):
    if model=='rows':
        rows='SELECT w,id,tag,src,lo,hi,rev,branch FROM c WHERE w=?1 AND tag=?8'
    else:
        rows="SELECT j.w,j.id,json_extract(e.value,'$.tag') tag,json_extract(e.value,'$.src') src,json_extract(e.value,'$.lo') lo,json_extract(e.value,'$.hi') hi,json_extract(e.value,'$.rev') rev,json_extract(e.value,'$.branch') branch FROM cj j CROSS JOIN json_each(j.doc) e WHERE j.w=?1 AND json_extract(e.value,'$.tag')=?8"
    sql='WITH tags AS MATERIALIZED ('+rows+''') SELECT DISTINCT t.id FROM tags t CROSS JOIN obj o ON o.w=t.w AND o.id=t.id CROSS JOIN obj s ON s.w=t.w AND s.id=t.src
    WHERE t.lo<=?2 AND (t.hi IS NULL OR t.hi>?3) AND ((t.branch='main' AND t.rev<=?4) OR (t.branch=?5 AND t.rev<=?6)) AND (?7 OR (o.hidden=0 AND s.hidden=0)) ORDER BY t.id LIMIT ?9'''
    result=[x[0] for x in db.execute(sql,args(v)+(tag,cap+1))];return result[:cap],len(result)>cap

db=build(); v=View(); checks=0
def expect(a,b):
    global checks
    assert a==b,(a,b);checks+=1
for subject,p,expected in [('king','holds',[]),('queen','holds',['crown']),('court','governs',['vale']),('king','adheres',['faith']),('temple','located',[]),('guild','uses',['tech']),('forge','produces',['sword']),('sword','material',['ore']),('forge','tool',['hammer']),('scribe','authored',['work']),('copy','embodies',['work']),('islander','speaks',['speech']),('automaton','species',['species']),('court','signatory',['treaty']),('league','signatory',['treaty']),('guild','signatory',['treaty']),('treaty','obligation',['guild']),('household','tenure',['farm']),('miner','works_at',['guild']),('town','market',['market']),('culture','descended_culture',['oldculture'])]:expect([x[1] for x in facts(db,v,subject,p) if x[3]=='fact'],expected)
expect([x[1] for x in facts(db,View(day=19),'king','holds')],['crown'])
expect(facts(db,View(branch='fork'),'king','holds')[0][3],'claim')
expect(facts(db,View(branch='fork'),'king','speaks'),[])
expect(facts(db,v,'queen','adheres'),[])
expect(len(facts(db,View(author=True),'miner','works_at')),2)
expect(facts(db,View(world='other'),'king','holds'),[])
for ctx in [v,View(branch='fork'),View(day=19),View(rev=1),View(author=True)]:
    for tag in CLASSES:expect(classes(db,ctx,tag,'rows'),classes(db,ctx,tag,'json'))
expect(classes(db,v,'settlement','rows')[0],['farm','town'])
expect(classes(db,View(branch='fork'),'settlement','rows')[0],['town'])
expect(classes(db,View(branch='fork'),'manor','rows')[0],['farm','town'])
expect(classes(db,v,'secret_guild','rows')[0],[])
expect(db.execute('SELECT amount,state FROM val WHERE id="town"').fetchone(),(None,'unknown'))
db.execute('UPDATE obj SET label="Renamed town" WHERE w="vale" AND id="town"'); db.commit()
expect(classes(db,v,'town','rows')[0],['town']);expect(facts(db,v,'town','market')[0][2],'records')
# Archive/restore with SQLite backup, not Atlas archive.
copy=sqlite3.connect(':memory:');db.backup(copy);expect(classes(copy,v,'town','rows'),classes(db,v,'town','rows'));expect(copy.execute('PRAGMA foreign_key_check').fetchall(),[])
print('assertions',checks,'plus wrong endpoint/world/class and negative value rejection')
results=[]
for n in [1000,10000,50000]:
    db=build(n);r=classes(db,v,'guild','rows');j=classes(db,v,'guild','json');expect(r,j);expect(r[1],True)
    times={m:[] for m in ('rows','json')}
    for _ in range(9):
        for m in times:
            t=time.perf_counter_ns();classes(db,v,'guild',m);times[m].append((time.perf_counter_ns()-t)/1e6)
    results.append(dict(background=n,match_cap=100,partial=r[1],median_ms={m:round(statistics.median(t),3) for m,t in times.items()}));db.close()
print(json.dumps(dict(python=platform.python_version(),sqlite=sqlite3.sqlite_version,platform=platform.platform(),results=results),indent=2))
