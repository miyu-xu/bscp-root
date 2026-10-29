"""Bounded application command; not a gfxstream wire change or task scheduler."""
import re

# Filled only from a verified shader build before a runnable candidate is frozen.
SHADER_SHA256 = 'dda872e96a97f60e7ba6d1553fffbe89c41c5a4b7b2d0557e7a833df8d6aabbf'
UINT = r'(?:0|[1-9][0-9]{0,9})'

def parse(command):
    if not isinstance(command,str) or not re.fullmatch(r'VADD [1-4](?: '+UINT+r'){2,8}',command):
        raise ValueError('invalid VADD grammar')
    tokens=command.split(' ');n=int(tokens[1]);values=list(map(int,tokens[2:]))
    if len(values)!=2*n or any(v>0xffffffff for v in values):
        raise ValueError('invalid VADD length or uint32')
    return n,values[:n],values[n:]

def validate_response(command,response,shader_sha256=None):
    n,a,b=parse(command)
    if not isinstance(response,str) or len(response)>2048 or not response.endswith('\n') or response.count('\n')!=1:
        raise ValueError('unknown: incomplete VADD response')
    if response in ('VADD ERR GPU_DISABLED\n','VADD ERR INPUT\n') or re.fullmatch(r'VADD ERR PREPARE reason=(?:NO_HARDWARE_DEVICE|NO_HARDWARE_COMPUTE_QUEUE|NO_COHERENT_MEMORY) vk_result=NONE\n',response) or re.fullmatch(r'VADD ERR PREPARE reason=VULKAN_API vk_result=-?[1-9][0-9]*\n',response):
        return {'outcome':'FAILED','response':response.strip(),'reference_checked':False}
    if response.startswith('VADD UNKNOWN '):raise ValueError('unknown: Guest retained submission')
    expected_shader=SHADER_SHA256 if shader_sha256 is None else shader_sha256
    if not re.fullmatch('[0-9a-f]{64}',expected_shader):raise ValueError('shader identity not frozen')
    parts=response[:-1].split(' ')
    fields=['n','values','device','vendor','deviceid','type','queue','memory','flags','dispatch','fence','shader']
    if parts[:2]!=['VADD','OK'] or len(parts)!=len(fields)+2:raise ValueError('unknown: response shape')
    d={}
    for key,part in zip(fields,parts[2:]):
        k,sep,v=part.partition('=')
        if k!=key or not sep:raise ValueError('unknown: response key/order')
        d[k]=v
    for key in ('n','vendor','deviceid','type','queue','dispatch','fence'):
        if not re.fullmatch(UINT,d[key]) or int(d[key])>0xffffffff:raise ValueError('unknown: numeric metadata')
    for key,count in (('values',n),('memory',3),('flags',3)):
        if not re.fullmatch(UINT+'(?:,'+UINT+')*',d[key]):raise ValueError('unknown: vector metadata')
        d[key]=list(map(int,d[key].split(',')))
        if len(d[key])!=count or any(v>0xffffffff for v in d[key]):raise ValueError('unknown: vector size')
    if not re.fullmatch(r'(?:[0-9a-f]{2}){1,255}',d['device']):raise ValueError('unknown: device name')
    device=bytes.fromhex(d['device']).decode('utf-8')
    if '\x00' in device or int(d['n'])!=n or int(d['type']) not in (1,2) or d['dispatch']!='1' or d['fence']!='0' or d['shader']!=expected_shader:
        raise ValueError('unknown: GPU execution identity')
    if any((v&6)!=6 for v in d['flags']) or any(v>=32 for v in d['memory']):raise ValueError('unknown: coherent memory identity')
    expected=[(x+y)&0xffffffff for x,y in zip(a,b)]
    return {'outcome':'SUCCESS' if expected==d['values'] else 'FAILED','reference_checked':True,
            'expected':expected,'actual':d['values'],'device':device,'metadata':d}
