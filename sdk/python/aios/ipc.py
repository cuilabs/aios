"""IPC API"""

class IPCClient:
    """IPC client"""
    
    def __init__(self):
        pass
    
    def send(self, to: int, data: bytes) -> None:
        """Send message"""
        # TODO: Send IPC message
        pass
    
    def receive(self) -> dict:
        """Receive message"""
        # TODO: Receive IPC message
        return {"from": 0, "data": b""}

