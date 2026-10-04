#!/usr/bin/env python3
"""Verify archive entry safety, exact file set and per-file SHA-256."""
import argparse,hashlib,json,pathlib,zipfile
def verify(path):
    with zipfile.ZipFile(path) as z:
        names=z.namelist();assert len(names)==len(set(names)),'Duplicate ZIP entry'
        for n in names:
            p=pathlib.PurePosixPath(n);assert not p.is_absolute() and '..' not in p.parts and '\\' not in n,'Unsafe archive path'
        assert sum(i.file_size for i in z.infolist())<=256*1024*1024,'Archive expansion cap exceeded'
        manifests=[n for n in names if n.endswith('/manifest.json') and n.count('/')==1]
        assert len(manifests)==1,'Expected one portable package root manifest'
        prefix=manifests[0].rsplit('/',1)[0]+'/'
        m=json.loads(z.read(manifests[0]));assert m['candidate'] in ['0.1.1','0.1.2'] and m['protocol_version']==1
        actual={n[len(prefix):] for n in names if not n.endswith('/') and n!=manifests[0]}
        assert actual==set(m['files']),'Package manifest file-set mismatch'
        for n,h in m['files'].items():assert hashlib.sha256(z.read(prefix+n)).hexdigest()==h,'Package hash mismatch: '+n
        assert 'Cargo.lock' in actual and 'package-lock.json' in actual
        assert m['commit'] and m['branch'] and m['architecture'] and m['build_timestamp']
        assert m['official_release_signature'] is False
        return {'verified':True,'files':len(actual),'candidate':m['candidate'],'commit':m['commit'],'platform':m['platform'],'sha256':hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()}
def main():
    p=argparse.ArgumentParser();p.add_argument('archive',type=pathlib.Path);a=p.parse_args();print(json.dumps(verify(a.archive),indent=2))
if __name__=='__main__':main()
