"""One strict JSON contract for persisted input and receipts."""
import json

def pairs(items):
    result={}
    for key,value in items:
        if key in result:raise ValueError('duplicate JSON key: '+key)
        result[key]=value
    return result

def constant(value):
    raise ValueError('nonfinite JSON number: '+value)

def loads(value):
    return json.loads(value,object_pairs_hook=pairs,parse_constant=constant)
