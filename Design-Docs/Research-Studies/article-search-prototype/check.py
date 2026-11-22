import json, sqlite3, statistics, time

def run():
    c = sqlite3.connect(':memory:')
    c.execute('CREATE VIRTUAL TABLE mixed USING fts5(title,body,scope UNINDEXED)')
    c.execute('CREATE VIRTUAL TABLE public_only USING fts5(title,body)')
    visible = [('river','charter charter charter charter'),('charter','river river')]
    c.executemany('INSERT INTO public_only VALUES(?,?)',visible)
    c.executemany('INSERT INTO mixed VALUES(?,?,?)',[(a,b,'public') for a,b in visible])
    def mixed():
        return [x[0] for x in c.execute("SELECT title FROM mixed WHERE mixed MATCH ? AND scope='public' ORDER BY bm25(mixed,2,1,0),title",('river OR charter',))]
    def scoped():
        return [x[0] for x in c.execute('SELECT title FROM public_only WHERE public_only MATCH ? ORDER BY bm25(public_only,2,1),title',('river OR charter',))]
    before = mixed(); isolated_before = scoped()
    c.executemany('INSERT INTO mixed VALUES(?,?,?)',[('river','river','hidden')]*500)
    after = mixed(); isolated_after = scoped()
    assert set(before)==set(after) and before!=after and isolated_before==isolated_after
    result={'sqlite':sqlite3.sqlite_version,'ranking':{'visible_before':before,'visible_after_500_hidden':after,'isolated_before':isolated_before,'isolated_after':isolated_after},'corpora':[]}
    for n in (1000,10000,50000):
        d=sqlite3.connect(':memory:')
        d.execute('CREATE TABLE blocks(id INTEGER PRIMARY KEY, article INTEGER NOT NULL, text TEXT NOT NULL)')
        rows=[(i+1,1 if i<75 else i-73,'river charter' if i<75 or i%10==0 else 'old forest') for i in range(n)]
        d.executemany('INSERT INTO blocks VALUES(?,?,?)',rows)
        d.execute("CREATE VIRTUAL TABLE f USING fts5(text,content='blocks',content_rowid='id',tokenize='trigram')")
        d.execute("INSERT INTO f(f) VALUES('rebuild')")
        original='SELECT b.article,b.id FROM f JOIN blocks b ON b.id=f.rowid WHERE f MATCH ? ORDER BY b.article,b.id LIMIT 50'
        grouped='SELECT b.article,MIN(b.id) FROM f JOIN blocks b ON b.id=f.rowid WHERE f MATCH ? AND b.article>? GROUP BY b.article ORDER BY b.article LIMIT 50'
        def block_page():return d.execute(original,('"river"',)).fetchall()
        def article_page(after=0):return d.execute(grouped,('"river"',after)).fetchall()
        b=block_page(); first=article_page(); second=article_page(first[-1][0]); oracle=sorted({a for _,a,t in rows if 'river' in t})
        assert len({a for a,_ in b})==1 and [a for a,_ in first+second]==oracle[:100]
        assert len({a for a,_ in first+second})==len(first+second)
        def med(fn):
            samples=[]
            for _ in range(9):
                start=time.perf_counter_ns();fn();samples.append((time.perf_counter_ns()-start)/1e6)
            return round(statistics.median(samples),3)
        result['corpora'].append({'raw_blocks':n,'matching_articles':len(oracle),'block_page_distinct_articles':len({a for a,_ in b}),'article_page_distinct_articles':len(first),'second_page_count':len(second),'block_page_median_ms':med(block_page),'grouped_page_median_ms':med(article_page)})
    names=sqlite3.connect(':memory:')
    names.execute("CREATE VIRTUAL TABLE n USING fts5(label,tokenize='trigram')")
    names.executemany('INSERT INTO n VALUES(?)', [('Io',), ('Iona',), ('Élodie',), ('E\u0301lodie',), ('Straße',)])
    def indexed(term):
        return [x[0] for x in names.execute('SELECT label FROM n WHERE n MATCH ? ORDER BY rowid',('"'+term.replace('"','""')+'"',))]
    short=indexed('Io'); fallback=[x[0] for x in names.execute('SELECT label FROM n ORDER BY rowid') if 'io' in x[0].lower()]
    assert short==[] and fallback==['Io','Iona']
    import unicodedata
    composed=indexed('Élodie'); decomposed=indexed('E\u0301lodie')
    assert composed==['Élodie'] and decomposed==['E\u0301lodie']
    normalized=[x[0] for x in names.execute('SELECT label FROM n ORDER BY rowid') if unicodedata.normalize('NFC',x[0]).casefold()==unicodedata.normalize('NFC','Élodie').casefold()]
    assert len(normalized)==2
    result['names']={'two_character_fts':short,'two_character_substring_fallback':fallback,'composed_query':composed,'decomposed_query':decomposed,'normalized_candidate_matches':normalized,'utf8_lengths':[len(x.encode('utf-8')) for x in normalized]}
    return result

if __name__=='__main__':
    print(json.dumps(run(),indent=2,ensure_ascii=False))
