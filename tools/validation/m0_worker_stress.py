"""Repeat the real Worker lib tests; fail on any nondeterministic assertion."""
import argparse
import concurrent.futures
import json
import pathlib
import platform
import subprocess
import time

p=argparse.ArgumentParser()
p.add_argument('--output',default='worker-stress.json')
a=p.parse_args()
build=subprocess.run(['cargo','test','-p','boardsi-worker','--lib','--no-run',
                      '--locked','--message-format=json'],capture_output=True,text=True,check=True)
artifacts=[json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
binary=next(x['executable'] for x in artifacts if x.get('executable') and x.get('target',{}).get('kind')==['lib'])
results=[]
for parallel,count,threads in ((1,100,1),(8,500,2)):
    start=time.monotonic()
    def one(index):
        proc=subprocess.run([binary,'--test-threads='+str(threads)],capture_output=True,text=True)
        if proc.returncode:
            pathlib.Path(f'worker-stress-failure-{index}.log').write_text(proc.stdout+proc.stderr)
        return proc.returncode
    with concurrent.futures.ThreadPoolExecutor(max_workers=parallel) as pool:
        codes=list(pool.map(one,range(count)))
    results.append(dict(parallel_processes=parallel,test_threads=threads,runs=count,
                        passed=codes.count(0),failed=sum(c!=0 for c in codes),
                        wall_s=time.monotonic()-start))
evidence=dict(platform=platform.platform(),results=results)
pathlib.Path(a.output).write_text(json.dumps(evidence,indent=2)+'\n')
print(json.dumps(evidence,indent=2))
raise SystemExit(any(x['failed'] for x in results))
