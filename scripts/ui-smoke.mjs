import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';

const [api, tokenFile, output = 'docs/evidence'] = process.argv.slice(2);
if (!api || !tokenFile) throw new Error('Usage: node scripts/ui-smoke.mjs <loopback-api> <token-file> [output-dir]');
const token = (await fs.readFile(tokenFile, 'utf8')).trim();
await fs.mkdir(output, { recursive: true });
const browser = await chromium.launch({headless:true,executablePath:process.env.PORCH_BROWSER_PATH||undefined,args:['--no-sandbox']});
const page = await browser.newPage({viewport:{width:1600,height:1120}});
const faults = [];
page.on('pageerror', error => faults.push(error.message));
try {
  await page.goto(api);
  await page.getByLabel('Local operator token').fill('invalid');
  await page.getByRole('button',{name:'Open my node'}).click();
  await page.getByRole('alert').waitFor();
  await page.getByLabel('Local operator token').fill(token);
  await page.getByRole('button',{name:'Open my node'}).click();
  await page.getByRole('heading',{name:'Your computers, together.'}).waitFor();
  assert.equal(await page.locator('.porch-name').innerText().then(x=>x.includes('Oak Street')),true);
  await page.screenshot({path:path.join(output,'home.png'),fullPage:true});
  await page.getByRole('button',{name:'Models',exact:true}).click();
  await page.getByLabel('Exact model name, or sha256 for a hash job').fill('porch-mock');
  await page.getByLabel('Privacy boundary').selectOption('TRUSTED_PEERS');
  await page.getByLabel('Input',{exact:true}).fill('UI acceptance request');
  await page.getByRole('button',{name:'Submit request',exact:true}).click();
  await page.locator('.result pre').filter({hasText:'MOCK inference porch-mock'}).waitFor();
  await page.screenshot({path:path.join(output,'models.png'),fullPage:true});
  await page.getByRole('button',{name:'Dismiss',exact:true}).click();
  for (const name of ['People','Resources','Compute','Storage','Network','Authority','Ledger']) {
    await page.getByRole('button',{name:name,exact:true}).click();
    await page.getByRole('heading',{name:name,exact:true,level:1}).waitFor();
    assert.equal(await page.getByRole('alert').count(),0,`${name} showed a runtime error`);
  }
  const downloadPromise=page.waitForEvent('download');
  await page.getByRole('button',{name:'Export visible events'}).click();
  const download=await downloadPromise;assert.equal(download.suggestedFilename(),'porch-ledger.json');
  assert.equal(await page.evaluate(()=>localStorage.length+sessionStorage.length),0);
  assert.equal((await page.locator('body').innerText()).includes(token),false);
  await page.getByRole('button',{name:'Home',exact:true}).click();
  await page.setViewportSize({width:390,height:844});
  await page.screenshot({path:path.join(output,'mobile.png'),fullPage:true});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+2),true,'mobile page overflows viewport');
  await page.getByRole('button',{name:'End operator session'}).click();
  await page.getByLabel('Local operator token').waitFor();
  assert.deepEqual(faults,[]);
  await fs.writeFile(path.join(output,'ui-receipt.json'),JSON.stringify({passed:true,browser:`Chromium headless ${browser.version()}`,scope:'real daemon UI, explicit deterministic mock model',checks:['operator refusal and login','actual remote job through UI','nine working navigation surfaces','ledger export','credentials remain in memory','390px responsive layout','logout','no page exceptions'],screenshots:['home.png','models.png','mobile.png']},null,2)+'\n');
  process.stdout.write('UI acceptance passed: 8 checks, 3 screenshots.\n');
} finally {await browser.close();}
