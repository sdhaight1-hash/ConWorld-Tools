function verify() {
  const checks=[];
  const check=(name,condition)=>{if(!condition)throw Error(name);checks.push(name);};
  const fails=(fn,message)=>{try{fn();return false;}catch(e){return e.message===message;}};
  const clone=x=>JSON.parse(JSON.stringify(x));
  // Model the observed top-level-only Query match in fresh_elements. This is
  // not a Rust execution and deliberately covers only these filter shapes.
  const leaf={op:"ParticipantEvents",target:"original"};
  const nested={op:"And",children:[{op:"Kind",kind:"event"},leaf]};
  function observedRebind(filter,source,destination) {
    const out=clone(filter);
    if(["ParticipantEvents","EventRole","RelationshipPath","WithinPlace","Backlinks"].includes(out.op)
       &&out.target===source)out.target=destination;
    return out;
  }
  check("observed top-level match rewrites supported self-like leaf",observedRebind(leaf,"original","new").target==="new");
  check("observed top-level match leaves nested supported filter unchanged",observedRebind(nested,"original","new").children[1].target==="original");
  const entities={original:"person",new:"person",city:"place",charter:"source"};
  // Candidate explicit intent, never automatic rebind by string equality.
  const definitions=new Map([
    ["profile@1",{version:1,blocks:[{id:"bio",text:"",audience:"author"},{id:"events",filter:{op:"And",children:[{op:"Kind",kind:"event"},{op:"ParticipantEvents",target:{self:"person"}}]}},{id:"literal",target:{literal:"original"}},{id:"citation",target:{literal:"charter"}}]}],
    ["profile@2",{version:2,blocks:[{id:"bio",text:"Biography prompt",audience:"author"}]}]
  ]);
  let serial=0;
  function resolveFilter(f,dest){
    if(f.children)return {...f,children:f.children.map(c=>resolveFilter(c,dest))};
    return f.target?{...f,target:resolveRef(f.target,dest)}:{...f};
  }
  function resolveRef(ref,dest){
    if(ref.self){if(entities[dest]!==ref.self)throw Error("incompatible slot");return dest;}
    if(!entities[ref.literal])throw Error("unavailable literal");
    return ref.literal;
  }
  function instance(pin,dest){
    const source=definitions.get(pin);if(!source)throw Error("unknown pin");
    // Resolve before creating any instance IDs; this is a pure model, not persisted.
    const blocks=source.blocks.map(b=>({...clone(b),...(b.filter?{filter:resolveFilter(b.filter,dest)}:{}),...(b.target?{target:resolveRef(b.target,dest)}:{})}));
    return {pin,dest,audience:"author",blocks:blocks.map(b=>({...b,id:"instance-"+(++serial)}))};
  }
  const v1=instance("profile@1","new"),original=JSON.stringify(definitions.get("profile@1"));
  check("explicit nested self slot resolves destination",v1.blocks[1].filter.children[1].target==="new");
  check("literal same-as-origin reference is preserved",v1.blocks[2].target==="original");
  check("independent source reference preserved",v1.blocks[3].target==="charter");
  check("incompatible self slot rejected",fails(()=>instance("profile@1","city"),"incompatible slot"));
  check("unknown definition pin rejected",fails(()=>instance("profile@0","new"),"unknown pin"));
  const more=instance("profile@1","new"),v2=instance("profile@2","new");
  check("fresh IDs across instantiations",!more.blocks.some(b=>v1.blocks.some(a=>a.id===b.id)));
  check("saved template version and existing instance unchanged",v1.pin==="profile@1"&&v1.blocks[0].text===""&&v2.blocks[0].text==="Biography prompt"&&JSON.stringify(definitions.get("profile@1"))===original);
  check("template instance defaults private",v1.audience==="author");
  // Separate draft and release in a hypothetical single-entry optimistic store.
  let state={rev:1,draft:"first text",release:"first text",history:[{rev:1,draft:"first text"}]};
  function mutate(expected,action,value){
    if(expected!==state.rev)throw Error("stale revision");
    const next=clone(state);next.rev++;
    if(action==="draft")next.draft=value;
    else if(action==="release")next.release=next.draft;
    else if(action==="revoke")next.release=null;
    else if(action==="restore"){const past=next.history.find(h=>h.rev===value);if(!past)throw Error("missing history");next.draft=past.draft;}
    else throw Error("unknown command");
    next.history.push({rev:next.rev,draft:next.draft});state=next;
    return next.rev;
  }
  const rev2=mutate(1,"draft","new private draft");
  check("draft save does not replace published text",state.release==="first text"&&state.draft==="new private draft");
  const saved=JSON.stringify(state);
  check("stale release rejected without state mutation",fails(()=>mutate(1,"release"),"stale revision")&&JSON.stringify(state)===saved);
  const rev3=mutate(rev2,"release");
  check("explicit release publishes current draft",state.release==="new private draft");
  const rev4=mutate(rev3,"revoke");
  const rev5=mutate(rev4,"restore",1);
  check("restoring draft history does not undo revocation",state.release===null&&state.draft==="first text");
  check("restore adds new revision and preserves recorded history",rev5===5&&state.history[1].draft==="new private draft");
  // Illustrative bounded block embedding, distinct from nonrecursive links.
  // Each reference is resolved in the same selected scope; no scope override.
  const pages={
    A:{released:true,audience:"public",blocks:[{id:"a",text:"A text"},{id:"r1",embed:"B"},{id:"r2",embed:"B"},{id:"r3",embed:"S",caption:"secret identity"}]},
    B:{released:true,audience:"public",blocks:[{id:"b",text:"B text"}]},
    S:{released:true,audience:"author",blocks:[{id:"s",text:"SECRET-XYZ"}]},
    C:{released:true,audience:"public",blocks:[{id:"c",embed:"D"}]},
    D:{released:true,audience:"public",blocks:[{id:"d",embed:"C"}]},
    X:{released:true,audience:"public",blocks:[{id:"x",embed:"B",audience:"author"}]}
  };
  function project(entry,scope,limit=16){
    let remaining=limit;
    const path=new Set();
    const allow=a=>a!=="author"||scope==="author";
    function visit(key,depth){
      const p=pages[key];if(!p||(!p.released&&scope!=="author")||!allow(p.audience))return [];
      if(path.has(key))throw Error("embed cycle");
      if(depth>4)throw Error("embed depth");
      path.add(key);
      let result=[];
      for(const b of p.blocks){
        if(!allow(b.audience))continue;
        if(--remaining<0)throw Error("embed budget");
        if(b.embed)result.push(...visit(b.embed,depth+1));
        else result.push({entry:key,block:b.id,text:b.text});
      }
      path.delete(key);
      return result;
    }
    return visit(entry,0);
  }
  const publicPage=project("A","public");
  check("shared source can appear twice without false cycle",publicPage.filter(b=>b.block==="b").length===2);
  check("private source and its reference caption omitted",!JSON.stringify(publicPage).includes("SECRET")&&!JSON.stringify(publicPage).includes("secret identity"));
  check("author sees private source under author scope",project("A","author").some(b=>b.block==="s"));
  check("hidden host reference does not include public source",project("X","public").length===0);
  check("cyclic embedding rejects without recursive expansion",fails(()=>project("C","public"),"embed cycle"));
  check("shared expansion uses global work budget",fails(()=>project("A","public",3),"embed budget"));
  pages.B.released=false;
  check("source revocation removes both instances on recompile",project("A","public").every(b=>b.entry!=="B"));
  pages.B.released=true;
  check("original public output reproducible under same state",JSON.stringify(project("A","public"))===JSON.stringify(publicPage));
  return {passed:checks.length,checks,notes:["Source-shaped top-level matching model, not actual Rust execution.","Candidate template/draft/release/embed rules intentionally synthetic.","No runtime timing or native application assertions."]};
}
console.log(JSON.stringify(verify(), null, 2));
