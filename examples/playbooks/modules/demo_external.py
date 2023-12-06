#!/usr/bin/env python3

import json
import sys
import fileinput

data = json.loads("\n".join([ x for x in fileinput.input() ]))
result = {}
data_a = int(data["a"])
data_b = int(data["b"])

result['sum'] = data_a + data_b
result['difference'] = data_a - data_b

msg = json.dumps(result)

print(msg)
sys.exit(0)