#!/usr/bin/env python3

import json
import sys

data = "\n".join(sys.stdin.readlines())
data = json.loads(data)

result = {}

# any non-zero exit will use 'msg' as an error message if the output is JSON
should_fail = result.get('simulate_failure', False)
if should_fail:
    result['msg'] = 'failing on request'
    msg = json.dumps(result)
    print(msg)
    sys.exit(1)

# otherwise print any json you want, but be sure to supress all non-JSON output
result['sum'] = data['a'] + data['b']
result['difference'] = data['a'] - data['b']
msg = json.dumps(result)

print(msg)
sys.exit(0)