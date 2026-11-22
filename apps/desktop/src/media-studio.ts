import {invoke} from "@tauri-apps/api/core";

export function openMediaStudio(parent:HTMLElement,request:{world_id:string;revision:number;day:number},editable:boolean,refresh:()=>Promise<void>,valid:()=>boolean){
  parent.querySelector("[data-media-studio]")?.remove();
  const host=document.createElement("details");host.dataset.mediaStudio="";host.className="query-panel";
  host.innerHTML=`<summary>World images and archive assets</summary><p class="hint">Import a still PNG up to 2 MiB and 2048 × 2048 pixels. Only sanitized pixels are retained; Canvas controls release and hotspot privacy.</p>
    <form class="editor-form"><label>World PNG image <input type="file" name="image" accept="image/png" required></label><button>Import image</button></form>
    <button type="button" data-list>List imported images</button><ul></ul><p role="status"></p>`;
  parent.prepend(host);
  const form=host.querySelector("form")!,input=form.elements.namedItem("image") as HTMLInputElement,status=host.querySelector<HTMLElement>("[role=status]")!;
  const submit=form.querySelector("button")!;submit.disabled=!editable;input.disabled=!editable;
  form.onsubmit=async event=>{
    event.preventDefault();if(!valid()||!editable||submit.disabled)return;const file=input.files?.[0];if(!file)return;
    if(file.size>2*1024*1024){status.textContent="Image exceeds 2 MiB.";return;}
    submit.disabled=true;
    try{const bytes=Array.from(new Uint8Array(await file.arrayBuffer()));if(!valid())return;await invoke("workspace_import_png",{request,name:file.name,bytes});if(valid())await refresh();}
    catch(error){if(valid())status.textContent=String(error);}
    finally{submit.disabled=false;}
  };
  host.querySelector<HTMLButtonElement>("[data-list]")!.onclick=async()=>{
    try{const assets=await invoke<{id:string;name:string;width:number;height:number}[]>("workspace_media_assets",{request});
      if(!valid())return;const list=host.querySelector("ul")!;list.replaceChildren();for(const asset of assets){const row=document.createElement("li");row.textContent=asset.name+" · "+asset.width+"×"+asset.height+" · "+asset.id;list.append(row);}
      status.textContent=assets.length+" imported images.";
    }catch(error){if(valid())status.textContent=String(error);}
  };
}
