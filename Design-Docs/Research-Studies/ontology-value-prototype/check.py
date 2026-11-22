"""Research #71 follow-up: compare plain typed concepts with profile contracts.
Independent Python fixture; not Atlas code, persistence, or a security proof.
Run: python check.py
"""
from dataclasses import dataclass
import json

@dataclass(frozen=True)
class Field:
    datatype: str
    unit: str = ''
    target: str = ''

@dataclass(frozen=True)
class Profile:
    base: str
    fields: tuple

# Stable IDs; profile versions are author metadata, not in-world claims.
profiles = {
    ('culture', 1): Profile('concept', (('language', Field('ref', target='language')),)),
    ('language', 1): Profile('concept', ()),
    ('site', 1): Profile('place', (('area', Field('quantity', 'ha')),)),
    ('material', 1): Profile('concept', ()),
    ('species', 1): Profile('concept', ()),
}
# obj: (world, stable id) -> base kind, profile pin, public visibility
objects = {
    ('vale','c1'): ('concept', ('culture',1), True),
    ('vale','l1'): ('concept', ('language',1), True),
    ('vale','l2'): ('concept', ('language',1), False),
    ('vale','m1'): ('concept', ('material',1), True),
    ('vale','p1'): ('place', ('site',1), True),
    ('vale','s1'): ('concept', ('species',1), True),
    ('other','l1'): ('concept', ('language',1), True),
}
# Canonical facts are explicit. Assignment of a profile creates no fact.
facts = []

def validate(ref, pin, name, value):
    kind, actual, _ = objects[ref]
    if actual != pin or pin not in profiles or profiles[pin].base != kind:
        raise ValueError('incompatible profile pin')
    fields = dict(profiles[pin].fields)
    if name not in fields:
        raise ValueError('undefined field')
    field = fields[name]
    if value is None:  # explicit unknown; missing field = no recorded assertion
        return
    if field.datatype == 'ref':
        if not isinstance(value, tuple) or len(value) != 2 or value[0] != ref[0] or value not in objects:
            raise ValueError('invalid reference')
        if objects[value][1][0] != field.target:
            raise ValueError('incompatible reference profile')
    elif field.datatype == 'quantity':
        if (not isinstance(value, tuple) or len(value) != 2 or
            type(value[0]) is not int or value[0] < 0 or value[1] != field.unit):
            raise ValueError('invalid quantity/unit')
    else:
        raise ValueError('unsupported datatype')

def record(ref, name, value, start, end, source, source_public=True):
    pin = objects[ref][1]
    validate(ref, pin, name, value)
    if end is not None and end <= start:
        raise ValueError('invalid interval')
    facts.append((ref,pin,name,value,start,end,source,source_public))

def read(ref, name, day, author=False):
    if ref not in objects or not (author or objects[ref][2]):
        return []
    result=[]
    for a,pin,n,v,lo,hi,src,public in facts:
        if a!=ref or n!=name or not (lo<=day and (hi is None or day<hi)):
            continue
        # Resolve historical contract, not latest name/version.
        field=dict(profiles[pin].fields)[name]
        if not author and (not public or (field.datatype=='ref' and v is not None and not objects[v][2])):
            continue
        result.append((v,src,pin))
    return result

checks=0
rejections=0
def eq(a,b):
    global checks
    assert a==b,(a,b); checks+=1

def reject(fn):
    global rejections
    before=len(facts)
    try: fn()
    except ValueError: pass
    else: raise AssertionError('accepted invalid data')
    eq(len(facts),before); rejections+=1

c=('vale','c1'); p=('vale','p1'); l=('vale','l1'); hidden=('vale','l2')
# Profile assignment is NOT language adoption, population, or a cultural identity fact.
eq(read(c,'language',0),[])
record(c,'language',l,0,20,'chronicle')
record(c,'language',hidden,20,None,'secret chronicle')
record(p,'area',(12,'ha'),0,20,'survey')
record(p,'area',None,20,None,'survey gap')
eq(read(c,'language',19),[(l,'chronicle',('culture',1))])
eq(read(c,'language',20),[])
eq(read(c,'language',20,True),[(hidden,'secret chronicle',('culture',1))])
eq(read(p,'area',19),[((12,'ha'),'survey',('site',1))])
eq(read(p,'area',20),[(None,'survey gap',('site',1))])
reject(lambda: record(c,'language',('vale','m1'),0,None,'bad'))
reject(lambda: record(c,'language',('other','l1'),0,None,'bad'))
reject(lambda: record(p,'area',(12,'kg'),0,None,'bad'))
reject(lambda: record(p,'area',(-1,'ha'),0,None,'bad'))
reject(lambda: record(p,'area',(True,'ha'),0,None,'bad'))
reject(lambda: record(c,'population',100,0,None,'bad'))
reject(lambda: record(c,'language',l,20,20,'bad'))
reject(lambda: validate(p,('culture',1),'language',l))
# A novel genre category: add metadata, do not change base kinds or validators.
profiles[('aetheric_drive',1)] = Profile('concept', (('medium',Field('ref',target='material')),))
objects[('vale','drive')]=('concept',('aetheric_drive',1),True)
eq(read(('vale','drive'),'medium',0),[])
record(('vale','drive'),'medium',('vale','m1'),0,None,'engineering notebook')
eq(read(('vale','drive'),'medium',1),[(('vale','m1'),'engineering notebook',('aetheric_drive',1))])
# New version changes units; require explicit reclassification, preserve prior interpretation.
profiles[('site',2)]=Profile('place',(('area',Field('quantity','m2')),))
reject(lambda: validate(p,('site',2),'area',(120000,'m2')))
objects[p]=('place',('site',2),True)
record(p,'area',(120000,'m2'),30,None,'new survey',False)
eq(read(p,'area',19),[((12,'ha'),'survey',('site',1))])
eq(read(p,'area',30),[(None,'survey gap',('site',1))])
eq(read(p,'area',30,True),[(None,'survey gap',('site',1)),((120000,'m2'),'new survey',('site',2))])
# Intentional limitation: overlapping unknown and known facts require a domain/cardinality policy;
# no automatic overwrite, conversion or exclusivity inference in this fixture.
print(json.dumps({'equality_assertions':checks,'invalid_cases_rejected':rejections,
 'records':len(facts),'profile_versions':len(profiles),
 'new_category_without_base_kind_change':True,'old_unit_interpretation_preserved':True},indent=2))
