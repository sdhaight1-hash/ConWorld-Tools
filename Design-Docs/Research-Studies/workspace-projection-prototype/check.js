function verify() {
  const results=[];
  function check(name, ok) { if (!ok) throw Error(name); results.push(name); }
  function allowed(scope,audience) { return scope==="author" || audience==="public" || scope===audience; }
  function restrict(a,b) { return a==="public"?b:b==="public"?a:a===b?a:"author"; }
  const tree={id:"root",audience:"public",children:[
    {id:"title",audience:"public",heading:"River Guild"},
    {id:"body",audience:"public",runs:[{text:"The river charter. "},{target:"charter",caption:"Charter"},{target:"secret",caption:"Secret vault"}]},
    {id:"sealed",audience:"author",children:[{id:"sealed-heading",audience:"public",heading:"Secret vault"},{id:"hidden-query",audience:"public",query:"never-read"}]},
    {id:"witness",audience:"witness",heading:"Witness testimony"},
    {id:"other",audience:"other",heading:"Other testimony"},
  ]};
  const visible=new Set(["charter"]);
  let calls=0;
  function page(node,scope) {
    if (!allowed(scope,node.audience)) return null;
    const out={id:node.id};
    if (node.children) out.children=node.children.map(c=>page(c,scope)).filter(Boolean);
    if (node.heading) out.heading=node.heading;
    if (node.runs) out.runs=node.runs.filter(r=>!r.target||scope==="author"||visible.has(r.target)).map(r=>({...r}));
    if (node.query) { calls++; if(scope!=="author")throw Error("unsupported public query");out.query="author-only";}
    return out;
  }
  function headings(node){return [...(node.heading?[node.heading]:[]),...(node.children||[]).flatMap(headings)];}
  const pub=page(tree,"public"), asWitness=page(tree,"witness"), asOther=page(tree,"other");
  check("public projection contains no private ids, captions or query",! /sealed|hidden-query|Secret vault|Witness testimony|Other testimony|never-read/.test(JSON.stringify(pub)));
  check("hidden subtree never evaluates query",calls===0);
  check("witness only sees own permitted section",headings(asWitness).join("|")==="River Guild|Witness testimony");
  check("other character only sees own section",headings(asOther).join("|")==="River Guild|Other testimony");
  check("author sees all headings",headings(page(tree,"author")).length===4&&calls===1);
  const moved={...tree.children[2].children[0],audience:restrict(tree.children[2].children[0].audience,tree.children[2].audience)};
  check("reparent preserves inherited restriction",page(moved,"public")===null&&moved.id==="sealed-heading");
  check("conflicting character scopes become author-only",restrict("witness","other")==="author");
  visible.delete("charter");
  check("revoked target and caption disappear on recompile",!JSON.stringify(page(tree,"public")).includes("Charter"));
  visible.add("charter");
  const blocks=page(tree,"public");
  const privateExtra={...tree,children:[...tree.children,{id:"hidden-X",audience:"author",heading:"CLASSIFIED NEW ENTRY"}]};
  check("hidden insertion leaves public DTO unchanged",JSON.stringify(blocks)===JSON.stringify(page(privateExtra,"public")));
  const rowSet=n=>Array.from({length:n},(_,i)=>({id:i+1,group:i%3===0?"House A":"House B",audience:i%7===0?"author":"public",start:i%11,end:i%5===0?12:null}));
  function select(rows,day,scope){return rows.filter(r=>allowed(scope,r.audience)&&r.start<=day&&(r.end===null||day<r.end)).map(({id,group})=>({id,group}));}
  function group(rows){return Object.fromEntries(["House A","House B"].map(g=>[g,rows.filter(r=>r.group===g).length]));}
  const metrics=[];
  for(const n of [1000,10000,50000]){
    const rows=rowSet(n), all=select(rows,12,"public"), first=all.slice(0,50), counts=group(all), partial=group(first);
    const list=first.map(r=>r.id), cards=first.map(r=>({key:r.id,title:r.group})).map(r=>r.key);
    check(`${n}: list/cards expose identical page ids`,JSON.stringify(list)===JSON.stringify(cards));
    check(`${n}: groups sum only authorized rows`,Object.values(counts).reduce((a,b)=>a+b,0)===all.length);
    check(`${n}: first-page group counts are not total`,Object.values(partial).reduce((a,b)=>a+b,0)===50&&all.length>50);
    const changed=[...rows,{id:n+1,group:"House A",audience:"author",start:0,end:null}];
    check(`${n}: hidden addition leaves selection and grouping identical`,JSON.stringify(select(changed,12,"public"))===JSON.stringify(all)&&JSON.stringify(group(select(changed,12,"public")))===JSON.stringify(counts));
    check(`${n}: end-exclusive dates respected`,all.every(r=>rows[r.id-1].end===null));
    metrics.push({n,visible:all.length,full:counts,first:partial});
  }
  const existing=[{id:"first",runs:[{text:"A😀 "},{target:"charter",caption:"Charter"},{text:"."}]}];
  const inserted=[{...existing[0],runs:[{text:"New prefix: "},...existing[0].runs]}];
  check("structured link survives Unicode prefix insertion without byte-offset retargeting",inserted[0].id===existing[0].id&&inserted[0].runs[2].target==="charter");
  return {passed:results.length,checks:results,metrics};
}
console.log(JSON.stringify(verify(), null, 2));
