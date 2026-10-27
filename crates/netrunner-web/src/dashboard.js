(()=>{
const $=id=>document.getElementById(id),dl=[],ul=[];let es=null;
const order=["locating","selecting","latency","download","upload","jitter"];
function phase(p){const k=p==="servers"?"selecting":p,i=order.indexOf(k);
 document.querySelectorAll("#phases li").forEach(li=>{const j=order.indexOf(li.dataset.p);
 li.className=j<i?"done":j===i?"on":""});}
function draw(){const c=$("chart"),x=c.getContext("2d"),w=c.width,h=c.height;x.clearRect(0,0,w,h);
 const max=Math.max(10,...dl,...ul)*1.1;
 [[dl,"#00ff80"],[ul,"#ff2bd6"]].forEach(([a,col])=>{if(a.length<2)return;x.strokeStyle=col;x.lineWidth=2;x.beginPath();
 a.forEach((v,i)=>{const px=i/(a.length-1)*w,py=h-v/max*h;i?x.lineTo(px,py):x.moveTo(px,py)});x.stroke()});}
const n=(v,d=1)=>Number(v).toFixed(d);
function on(name,fn){es.addEventListener(name,e=>fn(JSON.parse(e.data)))}
function fin(){es&&es.close();es=null;$("run").disabled=false}
$("run").onclick=()=>{
 dl.length=ul.length=0;draw();$("run").disabled=true;$("verdict").textContent="";$("status").textContent="Starting…";
 ["dl","ul","ping","jit"].forEach(i=>$(i).textContent="–");
 es=new EventSource("/api/run");
 on("status",d=>$("status").textContent=d.message);
 on("busy",d=>{$("status").textContent=d.message;fin()});
 on("phase",d=>{$("status").textContent=d.title;phase(d.phase)});
 on("download",d=>{$("dl").textContent=n(d.mbps);dl.push(d.mbps);draw()});
 on("upload",d=>{$("ul").textContent=n(d.mbps);ul.push(d.mbps);draw()});
 on("download_done",d=>$("dl").textContent=n(d.mbps));
 on("upload_done",d=>$("ul").textContent=n(d.mbps));
 on("latency",d=>$("ping").textContent=n(d.avg_ms,0));
 on("jitter",d=>$("jit").textContent=n(d.jitter_ms));
 on("done",d=>{const r=d.result;$("dl").textContent=n(r.download_mbps);$("ul").textContent=n(r.upload_mbps);
  $("ping").textContent=n(r.ping_ms,0);$("jit").textContent=n(r.jitter_ms);
  $("verdict").textContent="Connection quality: "+d.quality;$("verdict").style.color=d.color;
  $("status").textContent="Done";document.querySelectorAll("#phases li").forEach(l=>l.className="done");fin();
  const tb=$("history"),tr=document.createElement("tr");
  [r.timestamp.slice(0,16).replace("T"," "),n(r.download_mbps),n(r.upload_mbps),n(r.ping_ms,0),d.quality,r.server_location]
   .forEach((t,i)=>{const td=document.createElement("td");td.textContent=t;if(i===4)td.style.color=d.color;tr.append(td)});
  if(tb.firstElementChild&&tb.firstElementChild.firstElementChild.colSpan>1)tb.textContent="";
  tb.prepend(tr);});
 es.onerror=()=>{if(es){$("status").textContent="Connection closed";fin()}};
};
})();
