from bound_owner import BoundInstance as SharedBoundInstance

class BoundInstance(SharedBoundInstance):
    def __init__(self,expected,inventory,controller_pid,ops,clock=__import__("time").monotonic,sleep=__import__("time").sleep,**kwargs):
        super().__init__(expected,inventory,controller_pid,ops=ops,handle_field="retained_audit_handles",clock=clock,sleep=sleep,**kwargs)
