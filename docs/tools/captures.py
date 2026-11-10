"""Captures d'écran de Bandua en mode démo, dans les trois langues.

    pip install playwright pillow && playwright install chromium
    python docs/tools/captures.py          # fr en es
    python docs/tools/annoter.py           # cadres et zooms dans docs/guide/

Les captures brutes et la position des éléments sont écrites dans docs/tools/out/.
"""
import sys, os, asyncio
from playwright.async_api import async_playwright
import pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/"docs"/"tools"/"out"
URL=(ROOT/"ui"/"index.html").as_uri()
CHROME=os.environ.get("CHROME")  # chemin de Chrome, sinon Chromium de Playwright
FIELDS={"fr":dict(project="Blade Runner",day="J03",operator="Domingo Olmo Martin",notes="Flare léger sur C004, pluie artificielle sur C007"),
        "en":dict(project="Blade Runner",day="D03",operator="Domingo Olmo Martin",notes="Slight flare on C004, rain towers on C007"),
        "es":dict(project="Blade Runner",day="D03",operator="Domingo Olmo Martin",notes="Ligero flare en C004, lluvia artificial en C007")}
SEL={"src":".left > div:nth-child(1)","dests":".left > div:nth-child(2)","ident":".left > div:nth-child(3)","fname":".fname",
 "algo":".algo","reread":".reread","reverifyBtn":"#reverifyBtn","lang":"header #lang","aboutBtn":"#aboutBtn","chip":"#chip",
 "ring":".ring","metrics":".metrics","bars":".bars","cur":"#cur","dash":".dash","clips":"#clipsWrap","log":"#log",
 "go":"#go","footMsg":"#footMsg","openPdf":"#openPdf","reveal":"#reveal","banner":"#banner .banner","mclose":"#mclose",
 "warn":".box .warn","activeRow":"tr.active","errRow":"tr:has(.st.err)","dest1":"#dests .dest:first-child","dest2":"#dests .dest:nth-child(2)",
 "addDest":"#addDest","card":"#card","camera":"#camera","notes":"#notes"}
import json
async def shot(page,name):
    await page.wait_for_timeout(1500); await page.screenshot(path=str(OUT/f"{name}.png"))
    r={}
    for k,q in SEL.items():
        b=await page.evaluate("q=>{const e=document.querySelector(q);if(!e)return null;const r=e.getBoundingClientRect();if(!r.width)return null;return [r.left,r.top,r.right,r.bottom]}",q)
        if b: r[k]=[round(v*1.5) for v in b]
    t=await page.evaluate("()=>{const e=document.getElementById('footMsg');if(!e.firstChild)return null;const g=document.createRange();g.selectNodeContents(e);const r=g.getBoundingClientRect();return [r.left,r.top,r.right,r.bottom]}")
    if t: r["footText"]=[round(v*1.5) for v in t]
    json.dump(r,open(OUT/f"{name}.json","w"))
async def fresh(b,lang,accepted=True):
    ctx=await b.new_context(viewport={"width":1280,"height":820},device_scale_factor=1.5,locale=lang)
    page=await ctx.new_page()
    await page.clock.install()
    init=f"localStorage.setItem('bandua.lang',JSON.stringify('{lang}'));"
    if accepted: init+="localStorage.setItem('bandua.accepted.v1','true');"
    await page.add_init_script(init)
    await page.goto(URL)
    await page.add_style_tag(content=".demo{display:none!important}")
    return page
async def fill(page,lang):
    for k,v in FIELDS[lang].items(): await page.fill("#"+k,v)
async def prepare(page,lang):
    await page.click("#src"); await page.clock.run_for(300)
    await page.click("#addDest"); await page.clock.run_for(200)
    await page.click("#addDest"); await page.clock.run_for(200)
    await fill(page,lang); await page.clock.run_for(200)
    await page.evaluate("document.activeElement.blur()")
async def main(lang):
    async with async_playwright() as p:
        b=await p.chromium.launch(executable_path=CHROME) if CHROME else await p.chromium.launch()
        page=await fresh(b,lang,accepted=False); await page.clock.run_for(300); await shot(page,f"welcome_{lang}")
        await page.click("#mclose"); await page.clock.run_for(200); await shot(page,f"empty_{lang}")
        await prepare(page,lang); await shot(page,f"ready_{lang}")
        await page.click("#go"); await page.clock.run_for(15000); await shot(page,f"offload_{lang}")
        for _ in range(3000):
            await page.clock.run_for(80)
            if await page.evaluate("document.querySelector('#banner .banner')!==null"): break
        await page.clock.run_for(300); await page.evaluate("document.getElementById('clipsWrap').scrollTop=0"); await shot(page,f"verified_{lang}")
        await page.click("#aboutBtn"); await page.clock.run_for(200); await shot(page,f"about_{lang}"); await page.click("#mclose")
        page=await fresh(b,lang); await page.clock.run_for(300)
        await page.click("#reverifyBtn"); await page.clock.run_for(2500); await shot(page,f"reverify_run_{lang}")
        for _ in range(3000):
            await page.clock.run_for(80)
            if await page.evaluate("document.querySelector('#banner .banner')!==null"): break
        await page.clock.run_for(300); await shot(page,f"reverify_err_{lang}")
        await b.close()
OUT.mkdir(exist_ok=True)
for l in (sys.argv[1:] or ["fr","en","es"]): asyncio.run(main(l))
