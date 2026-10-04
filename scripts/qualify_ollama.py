#!/usr/bin/env python3
"""Opt-in live provider suite against an existing node. Never uses mock fallback."""
import argparse,json,os,pathlib,subprocess,time
ROOT=pathlib.Path(__file__).resolve().parents[1]
def main():
    p=argparse.ArgumentParser();p.add_argument('--data',type=pathlib.Path,required=True);p.add_argument('--api',default='http://127.0.0.1:7331');p.add_argument('--model');p.add_argument('--cli',type=pathlib.Path,default=ROOT/'target/debug'/('porch.exe' if os.name=='nt' else 'porch'));p.add_argument('--out',type=pathlib.Path,required=True);a=p.parse_args()
    def run(*args):
        r=subprocess.run([str(a.cli),'--data',str(a.data),'--api',a.api,*args],capture_output=True,text=True,timeout=90)
        try:return r.returncode,json.loads(r.stdout)
        except json.JSONDecodeError:return r.returncode,{'reason':'CLI refused; inspect local operator diagnostics'}
    code,scan=run('models','scan');scenarios=[]
    def add(id,expectation,observation,result):scenarios.append({'id':id,'expectation':expectation,'observation':observation,'result':result,'timestamp':int(time.time()),'evidence_file':a.out.name,'peers':[]})
    reachable=scan.get('currently_reachable') is True
    add('live_provider','Local Ollama tags endpoint responds',{'reachable':reachable,'scan':scan},'PASS' if reachable else 'SKIPPED_ENVIRONMENT' if code==0 else 'FAIL')
    model=a.model
    if reachable and model:
        present=any(m['model']==model for m in scan.get('inventory',[]));add('selected_model','Chosen model is presently installed',{'model':model,'present':present},'PASS' if present else 'FAIL')
        if present:
            code,result=run('models','verify',model);receipt=result.get('receipt',{}).get('payload',{})
            ok=code==0 and result.get('status')=='COMPLETED' and receipt.get('provider')=='ollama' and not receipt.get('output',{}).get('mock',True)
            add('live_inference','Fixed public probe returns real Ollama output and signed receipt',result,'PASS' if ok else 'FAIL')
    else:add('live_inference','Explicit --model selects real weights',{'model':model},'SKIPPED_ENVIRONMENT')
    for name in ['provider_process_death','model_unload','model_inventory_change']:
        add(name,'Repeat operator-controlled live procedure in docs/OLLAMA_QUALIFICATION.md',{'live_external_test_required':True},'UNVERIFIED')
    a.out.parent.mkdir(parents=True,exist_ok=True);a.out.write_text(json.dumps({'suite':'real Ollama opt-in','scenarios':scenarios,'physical_qualification':'UNVERIFIED'},indent=2)+'\n')
    print(a.out)
    if any(s['result']=='FAIL' for s in scenarios):raise SystemExit(1)
if __name__=='__main__':main()
