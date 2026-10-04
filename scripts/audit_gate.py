#!/usr/bin/env python3
"""Keep findings visible and reject any new or newly reachable advisory.

The two reviewed Hickory findings are constrained to this exact mDNS dependency
and feature graph. No global advisory ignores, feature expansion, or major
dependency upgrades are performed by this gate.
"""
import argparse,hashlib,json,os,pathlib,subprocess,tomllib
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'docs/receipts/qualification'
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--offline',action='store_true');args=parser.parse_args()
    result=subprocess.run(['cargo','audit','--json',*(['--no-fetch'] if args.offline else [])],cwd=ROOT,capture_output=True,text=True,timeout=90)
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/'cargo-audit.json').write_text(result.stdout)
    (OUT/'cargo-audit.log').write_text(result.stderr)
    audit=json.loads(result.stdout)
    if audit.get('error'):raise SystemExit('Audit tool failed; no clean audit claim is allowed')
    graph=subprocess.check_output(['cargo','tree','-i','hickory-proto','-e','features'],cwd=ROOT,text=True)
    (OUT/'hickory-features.txt').write_text(graph)
    assert 'dnssec' not in graph.lower(),'DNSSEC was enabled: reviewed applicability no longer holds'
    locks=tomllib.loads((ROOT/'Cargo.lock').read_text())['package']
    assert next(p['version'] for p in locks if p['name']=='libp2p-mdns')=='0.48.0'
    assert next(p['version'] for p in locks if p['name']=='hickory-proto')=='0.25.2'
    cargo_home=pathlib.Path(os.environ.get('CARGO_HOME',pathlib.Path.home()/'.cargo'))
    sources=list(cargo_home.glob('registry/src/*/libp2p-mdns-0.48.0/src/behaviour/iface'))
    assert sources,'Build dependencies before running the source applicability check'
    dns=(sources[0]/'dns.rs').read_text();query=(sources[0]/'query.rs').read_text()
    assert 'BinEncoder' not in dns and 'BinEncoder' not in query
    assert 'Message::from_vec' in query
    assert 'append_qname' in dns and 'out.extend_from_slice' in dns
    expected={'RUSTSEC-2026-0118':'unreachable: DNSSEC feature and validation handle are absent',
              'RUSTSEC-2026-0119':'unreachable: mDNS decodes with Hickory, but encodes manually without BinEncoder'}
    findings=[]
    for finding in audit.get('vulnerabilities',{}).get('list',[]):
        advisory=finding['advisory'];id=advisory['id']
        assert id in expected and finding['package']['name']=='hickory-proto','Unreviewed advisory: '+id
        findings.append({'id':id,'package':'hickory-proto 0.25.2','classification':'unreachable_in_current_configuration','reason':expected[id],'upgrade_required_if_scope_changes':True})
    warnings=audit.get('warnings',{})
    for category,items in warnings.items():
        for item in items:
            assert category=='unmaintained' and item['package']['name']=='paste','Unreviewed audit warning'
            findings.append({'id':item['advisory']['id'],'package':'paste 1.0.15','classification':'build_time_maintenance_risk','reason':'Transitive macro dependency. Retained for this bounded candidate; replace through upstream dependency update.'})
    report={'candidate':'0.1.1','audit_tool':'cargo audit','database_mode':'existing snapshot; no freshness claim' if args.offline else 'fetched for this run','raw_vulnerabilities':audit.get('vulnerabilities',{}).get('count'),
            'gate':'PASS_WITH_REVIEWED_FINDINGS','findings':findings,'independent_security_review':'UNVERIFIED',
            'mdns_source_hashes':{name:hashlib.sha256((sources[0]/name).read_bytes()).hexdigest() for name in ['dns.rs','query.rs']}}
    (OUT/'supply-chain-review.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))
if __name__=='__main__':main()
