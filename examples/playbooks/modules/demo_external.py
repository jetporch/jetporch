#!/usr/bin/env python3

import json
import sys
import fileinput

data = json.loads("\n".join([ x for x in fileinput.input() ]))
result = {}
result['sum'] = data["a"] + data["b"]
result['difference'] = data["a"] - data["b"]

msg = json.dumps(result)

print(msg)
sys.exit(0)